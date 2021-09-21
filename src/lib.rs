mod data_checksum;
mod extension;
mod header;
mod puzzle;
mod puzzle_buffer;
mod puzzle_type;
mod solution_state;

pub use puzzle::Puzzle;
pub use puzzle_type::PuzzleType;
pub use solution_state::SolutionState;

/*

    def load(self, data):
        ext_cksum = {}
        while s.can_unpack(EXTENSION_HEADER_FORMAT):
            code, length, cksum = s.unpack(EXTENSION_HEADER_FORMAT)
            ext_cksum[code] = cksum
            # extension data is represented as a null-terminated string,
            # but since the data can contain nulls we can't use read_string
            self.extensions[code] = s.read(length)
            s.read(1)  # extensions have a trailing byte
            # save the codes in order for round-tripping
            self._extensions_order.append(code)

        # sometimes there's some extra garbage at
        # the end of the file, usually \r\n
        if s.can_read():
            self.postscript = s.read_to_end()

        if cksum_gbl != self.global_cksum():
            raise PuzzleFormatError('global checksum does not match')
        if cksum_hdr != self.header_cksum():
            raise PuzzleFormatError('header checksum does not match')
        if cksum_magic != self.magic_cksum():
            raise PuzzleFormatError('magic checksum does not match')
        for code, cksum_ext in ext_cksum.items():
            if cksum_ext != data_cksum(self.extensions[code]):
                raise PuzzleFormatError(
                    'extension %s checksum does not match' % code
                )

    def save(self, filename):
        puzzle_bytes = self.tobytes()
        with open(filename, 'wb') as f:
            f.write(puzzle_bytes)

    def tobytes(self):
        s = PuzzleBuffer(encoding=self.encoding)
        # commit any changes from helpers
        for h in self.helpers.values():
            if 'save' in dir(h):
                h.save()

        # include any preamble text we might have found on read
        s.write(self.preamble)

        s.pack(HEADER_FORMAT,
               self.global_cksum(), ACROSSDOWN,
               self.header_cksum(), self.magic_cksum(),
               self.fileversion, self.unk1, self.scrambled_cksum,
               self.unk2, self.width, self.height,
               len(self.clues), self.puzzletype, self.solution_state)

        s.write(self.encode(self.solution))
        s.write(self.encode(self.fill))

        s.write_string(self.title)
        s.write_string(self.author)
        s.write_string(self.copyright)

        for clue in self.clues:
            s.write_string(clue)

        s.write_string(self.notes)

        # do a bit of extra work here to ensure extensions round-trip in the
        # order they were read. this makes verification easier. But allow
        # for the possibility that extensions were added or removed from
        # self.extensions
        ext = dict(self.extensions)
        for code in self._extensions_order:
            data = ext.pop(code, None)
            if data:
                s.pack(EXTENSION_HEADER_FORMAT, code,
                       len(data), data_cksum(data))
                s.write(data + b'\0')

        for code, data in ext.items():
            s.pack(EXTENSION_HEADER_FORMAT, code, len(data), data_cksum(data))
            s.write(data + b'\0')

        # postscript is initialized, read, and stored as bytes. In case it is
        # overwritten as a string, this try/except converts it back.
        try:
            postscript_bytes = self.encode(self.postscript)
        except AttributeError:
            postscript_bytes = self.postscript

        s.write(postscript_bytes)

        return s.tobytes()

    def encode(self, s):
        return s.encode(self.encoding, ENCODING_ERRORS)

    def encode_zstring(self, s):
        return self.encode(s) + b'\0'

    def version_tuple(self):
        return tuple(map(int, self.version.split(b'.')))

    def has_rebus(self):
        return self.rebus().has_rebus()

    def rebus(self):
        return self.helpers.setdefault('rebus', Rebus(self))

    def has_markup(self):
        return self.markup().has_markup()

    def markup(self):
        return self.helpers.setdefault('markup', Markup(self))

    def clue_numbering(self):
        numbering = DefaultClueNumbering(self.fill, self.clues, self.width, self.height)
        return self.helpers.setdefault('clues', numbering)

    def blacksquare(self):
        return BLACKSQUARE2 if self.puzzletype == PuzzleType.Diagramless else BLACKSQUARE

    def is_solution_locked(self):
        return bool(self.solution_state != SolutionState.Unlocked)

    def unlock_solution(self, key):
        if self.is_solution_locked():
            unscrambled = unscramble_solution(self.solution, self.width, self.height, key,
                                              ignore_chars=self.blacksquare())
            if not self.check_answers(unscrambled):
                return False

            # clear the scrambled bit and cksum
            self.solution = unscrambled
            self.scrambled_cksum = 0
            self.solution_state = SolutionState.Unlocked

        return True

    def lock_solution(self, key):
        if not self.is_solution_locked():
            # set the scrambled bit and cksum
            self.scrambled_cksum = scrambled_cksum(self.solution, self.width, self.height,
                                                   ignore_chars=self.blacksquare(), encoding=self.encoding)
            self.solution_state = SolutionState.Locked
            scrambled = scramble_solution(self.solution, self.width, self.height, key,
                                          ignore_chars=self.blacksquare())
            self.solution = scrambled

    def check_answers(self, fill):
        if self.is_solution_locked():
            scrambled = scrambled_cksum(fill, self.width, self.height,
                                        ignore_chars=self.blacksquare(), encoding=self.encoding)
            return scrambled == self.scrambled_cksum
        else:
            return fill == self.solution

    def header_cksum(self, cksum=0):
        return data_cksum(struct.pack(HEADER_CKSUM_FORMAT,
                          self.width, self.height, len(self.clues),
                          self.puzzletype, self.solution_state), cksum)

    def text_cksum(self, cksum=0):
        # for the checksum to work these fields must be added in order with
        # null termination, followed by all non-empty clues without null
        # termination, followed by notes (but only for version >= 1.3)
        if self.title:
            cksum = data_cksum(self.encode_zstring(self.title), cksum)
        if self.author:
            cksum = data_cksum(self.encode_zstring(self.author), cksum)
        if self.copyright:
            cksum = data_cksum(self.encode_zstring(self.copyright), cksum)

        for clue in self.clues:
            if clue:
                cksum = data_cksum(self.encode(clue), cksum)

        # notes included in global cksum starting v1.3 of format
        if self.version_tuple() >= (1, 3) and self.notes:
            cksum = data_cksum(self.encode_zstring(self.notes), cksum)

        return cksum

    def global_cksum(self):
        cksum = self.header_cksum()
        cksum = data_cksum(self.encode(self.solution), cksum)
        cksum = data_cksum(self.encode(self.fill), cksum)
        cksum = self.text_cksum(cksum)
        # extensions do not seem to be included in global cksum
        return cksum

    def magic_cksum(self):
        cksums = [
            self.header_cksum(),
            data_cksum(self.encode(self.solution)),
            data_cksum(self.encode(self.fill)),
            self.text_cksum()
        ]

        cksum_magic = 0
        for (i, cksum) in enumerate(reversed(cksums)):
            cksum_magic <<= 8
            cksum_magic |= (
                ord(MASKSTRING[len(cksums) - i - 1]) ^ (cksum & 0x00ff)
            )
            cksum_magic |= (
                (ord(MASKSTRING[len(cksums) - i - 1 + 4]) ^ (cksum >> 8)) << 32
            )

        return cksum_magic



*/

/*
    def test_diagramless_clue_numbering(self):
        p = puz.read('testfiles/nyt_diagramless.puz')
        clues = p.clue_numbering()
        self.assertEqual(len(p.clues), len(clues.across) + len(clues.down))
        self.assertTrue(len(p.clues) > 0)


    }
}

class PuzzleTests(unittest.TestCase):

    def test_clue_numbering(self):
        p = puz.read('testfiles/washpost.puz')
        clues = p.clue_numbering()
        self.assertEqual(len(p.clues), len(clues.across) + len(clues.down))
        self.assertTrue(len(p.clues) > 0)

    def test_diagramless_clue_numbering(self):
        p = puz.read('testfiles/nyt_diagramless.puz')
        clues = p.clue_numbering()
        self.assertEqual(len(p.clues), len(clues.across) + len(clues.down))
        self.assertTrue(len(p.clues) > 0)

    def test_extensions(self):
        p = puz.read('testfiles/nyt_rebus_with_notes_and_shape.puz')
        # We don't use assertIn for compatibility with Python 2.6
        self.assertTrue(puz.Extensions.Rebus in p.extensions)
        self.assertTrue(puz.Extensions.RebusSolutions in p.extensions)
        self.assertTrue(puz.Extensions.Markup in p.extensions)

    def test_rebus(self):
        p = puz.read('testfiles/nyt_rebus_with_notes_and_shape.puz')
        self.assertTrue(p.has_rebus())
        r = p.rebus()
        self.assertTrue(r.has_rebus())
        self.assertEqual(3, len(r.get_rebus_squares()))
        self.assertTrue(all(r.is_rebus_square(i)
                            for i in r.get_rebus_squares()))
        self.assertTrue(all('STAR' == r.get_rebus_solution(i)
                            for i in r.get_rebus_squares()))
        self.assertEqual(None, r.get_rebus_solution(100))
        # trigger save
        p.tobytes()

    def test_markup(self):
        p = puz.read('testfiles/nyt_rebus_with_notes_and_shape.puz')
        self.assertTrue(p.has_markup())
        m = p.markup()
        self.assertTrue(all(puz.GridMarkup.Circled == m.markup[i]
                            for i in m.get_markup_squares()))
        # trigger save
        p.tobytes()

        p = puz.read('testfiles/washpost.puz')
        self.assertFalse(p.has_markup())
        m = p.markup()
        self.assertFalse(m.has_markup())
        # trigger save
        p.tobytes()

    def test_puzzle_type(self):
        self.assertNotEqual(
            puz.read('testfiles/washpost.puz').puzzletype,
            puz.PuzzleType.Diagramless)
        self.assertNotEqual(
            puz.read('testfiles/nyt_locked.puz').puzzletype,
            puz.PuzzleType.Diagramless)
        self.assertEqual(
            puz.read('testfiles/nyt_diagramless.puz').puzzletype,
            puz.PuzzleType.Diagramless)

    def test_empty_puzzle(self):
        p = puz.Puzzle()
        self.assertRaises(puz.PuzzleFormatError, p.load, b'')

    def test_junk_at_end_of_puzzle(self):
        with open('testfiles/washpost.puz', 'rb') as fp:
            data = fp.read() + b'\r\n\r\n'
        p = puz.Puzzle()
        p.load(data)
        self.assertEqual(p.postscript, b'\r\n\r\n')

    def test_v1_4(self):
        p = puz.read('testfiles/nyt_v1_4.puz')
        p.tobytes()

    def test_v2_unicode(self):
        p = puz.read('testfiles/unicode.puz')
        # puzzle title contains emoji
        self.assertEqual(p.title, u'\u2694\ufe0f')
        self.assertEqual(p.encoding, 'UTF-8')
        p.tobytes()

    def test_v2_upgrade(self):
        p = puz.read('testfiles/washpost.puz')
        p.title = u'\u2694\ufe0f'
        p.version = b'2.0'
        p.fileversion = b'2.0\0'
        p.encoding = puz.ENCODING_UTF8
        data = p.tobytes()
        p2 = puz.load(data)
        self.assertEqual(p2.title, u'\u2694\ufe0f')

    def test_save_empty_puzzle(self):
        ''' confirm an empty Puzzle() can be saved to a file '''
        p = puz.Puzzle()
        with tempfile.NamedTemporaryFile(suffix='.puz') as tmp:
            p.save(tmp.name)
            p2 = puz.read(tmp.name)
            self.assertEqual(p.puzzletype, p2.puzzletype)
            self.assertEqual(p.version, p2.version)
            self.assertEqual(p.scrambled_cksum, p2.scrambled_cksum)

    def test_save_small_puzzle(self):
        ''' an example of creating a small 3x3 puzzle from scratch and writing
        to a file
        '''
        p = puz.Puzzle()
        with tempfile.NamedTemporaryFile(suffix='.puz') as tmp:
            p.title = 'Test Puzzle'
            p.author = 'Alex'
            p.height = 3
            p.width = 3
            p.solution = 'A' * 9
            p.clues = ['clue'] * 6
            p.fill = '-' * 9
            p.save(tmp.name)
            p2 = puz.read(tmp.name)
            self.assertEqual(p.title, p2.title)
            self.assertEqual(p.author, p2.author)
            self.assertEqual(p.solution, p2.solution)
            self.assertEqual(p.clues, p2.clues)
            self.assertEqual(p.fill, p2.fill)


class LockTests(unittest.TestCase):

    def test_scramble_functions(self):
        ''' tests some examples from the file format documentation wiki
        '''
        self.assertEqual('MLOOPKJ', puz.scramble_string('AEBFCDG', 1234))
        self.assertEqual('MOP..KLOJ',
                         puz.scramble_solution('ABC..DEFG', 3, 3, 1234))

        self.assertEqual('AEBFCDG', puz.unscramble_string('MLOOPKJ', 1234))
        self.assertEqual('ABC..DEFG',
                         puz.unscramble_solution('MOP..KLOJ', 3, 3, 1234))

        # rectangular example - tricky
        a = 'ABCD.EFGH.KHIJKLM.NOPW.XYZ'
        scrambled = puz.scramble_solution(a, 13, 2, 9721)
        unscrambled = puz.unscramble_solution(scrambled, 13, 2, 9721)
        self.assertEqual(a, unscrambled)

    def test_locked_bit(self):
        self.assertFalse(
            puz.read('testfiles/washpost.puz').is_solution_locked())
        self.assertTrue(
            puz.read('testfiles/nyt_locked.puz').is_solution_locked())

    def test_unlock(self):
        p = puz.read('testfiles/nyt_locked.puz')
        self.assertTrue(p.is_solution_locked())
        self.assertFalse(p.unlock_solution(1234))
        self.assertTrue(p.is_solution_locked())  # still locked
        self.assertTrue(p.unlock_solution(7844))
        self.assertFalse(p.is_solution_locked())  # unlocked!
        # We don't use assertIn for compatibility with Python 2.6
        self.assertTrue('LAKEONTARIO' in p.solution)

    def test_unlock_relock(self):
        with open('testfiles/nyt_locked.puz', 'rb') as fp:
            orig = fp.read()
        p = puz.read('testfiles/nyt_locked.puz')
        self.assertTrue(p.is_solution_locked())
        self.assertTrue(p.unlock_solution(7844))
        p.lock_solution(7844)
        new = p.tobytes()
        self.assertEqual(orig, new, 'nyt_locked.puz did not round-trip')

    def test_check_answers_locked(self):
        '''Verify that we can check answers even when the solution is locked
        '''
        p1 = puz.read('testfiles/nyt_locked.puz')
        p2 = puz.read('testfiles/nyt_locked.puz')
        p1.unlock_solution(7844)
        self.assertTrue(p2.is_solution_locked())
        self.assertTrue(p2.check_answers(p1.solution))

    def test_unlock_relock_diagramless(self):
        with open('testfiles/nyt_diagramless.puz', 'rb') as fp:
            orig = fp.read()
        p = puz.read('testfiles/nyt_diagramless.puz')
        self.assertTrue(p.is_solution_locked())
        self.assertTrue(p.unlock_solution(3285))
        self.assertFalse(p.is_solution_locked())
        p.lock_solution(3285)
        new = p.tobytes()
        self.assertEqual(orig, new, 'nyt_diagramless.puz did not round-trip')


class RoundtripPuzfileTests(unittest.TestCase):

    def __init__(self, filename):
        unittest.TestCase.__init__(self)
        self.filename = filename

    def runTest(self):
        try:
            with open(self.filename, 'rb') as fp_filename:
                orig = fp_filename.read()
                p = puz.read(self.filename)
                if (p.puzzletype == puz.PuzzleType.Normal):
                    clues = p.clue_numbering()
                    # smoke test the clue numbering while we're at it
                    self.assertEqual(
                        len(p.clues), len(clues.across) + len(clues.down),
                        'failed in %s' % self.filename)
                # this is the roundtrip
                new = p.tobytes()
                self.assertEqual(orig, new,
                                 '%s did not round-trip' % self.filename)
        except puz.PuzzleFormatError:
            args = (self.filename, sys.exc_info()[1].message)
            self.assertTrue(False, '%s threw PuzzleFormatError: %s' % args)


def tests_in_dir(directory):
    tests = []
    for path, _, _ in os.walk(directory):
        for filename in glob.glob(os.path.join(path, '*.puz')):
            tests.append(RoundtripPuzfileTests(filename))
    return tests


def suite():
    # suite consists of any test* method defined in PuzzleTests,
    # plus a round-trip test for each .puz file in ./testfiles/
    suite = unittest.TestSuite()
    loader = unittest.defaultTestLoader
    suite.addTests(loader.loadTestsFromTestCase(PuzzleTests))
    suite.addTests(loader.loadTestsFromTestCase(LockTests))
    suite.addTests(tests_in_dir('testfiles'))
    return suite


if __name__ == '__main__':
    print(__file__)
    result = unittest.TextTestRunner().run(suite())
    sys.exit(not result.wasSuccessful())
*/
