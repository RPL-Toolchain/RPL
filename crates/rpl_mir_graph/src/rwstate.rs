use rustc_index::Idx;

use crate::graph::Access;

/// Conceptually, this is like a `Vec<Vec<RWCState>>`. But the number of
/// RWC's can get very large, so it uses a more compact representation.
#[derive(Clone)]
pub(super) struct RWCStates<Local: Idx> {
    /// Total number of statements.
    statements: usize,
    /// Total number of locals.
    locals: usize,

    /// A compressed representation of `RWCState`s.
    ///
    /// Each word represents 4 different `RWCState`s packed together. Each packed
    /// RWCState is stored in 4 bits: a read bit, and a write bit.
    ///
    /// The data for each statement is contiguous and starts at a word boundary,
    /// so there might be an unused space left.
    words: Vec<u8>,
    /// Number of words per each statement.
    statement_words: usize,
    _marker: std::marker::PhantomData<Local>,
}

impl<Local: Idx> RWCStates<Local> {
    const RWC_READ: u8 = 0b0001;
    const RWC_WRITE: u8 = 0b0010;
    const RWC_CONSUME: u8 = 0b0100;
    const RWC_MASK: u8 = 0b0111;

    /// Size of packed RWC in bits.
    const RWC_BITS: usize = 4;
    /// Size of a word in bits.
    const WORD_BITS: usize = std::mem::size_of::<u8>() * 8;
    /// Number of packed RWCStates that fit into a single word.
    const WORD_RWC_COUNT: usize = Self::WORD_BITS / Self::RWC_BITS;

    pub(super) fn new(statements: usize, locals: usize) -> Self {
        let statement_words = locals.div_ceil(Self::WORD_RWC_COUNT);
        Self {
            statements,
            locals,
            statement_words,
            words: vec![0u8; statement_words * statements],
            _marker: std::marker::PhantomData,
        }
    }
    pub(super) fn num_statements(&self) -> usize {
        self.statements
    }
    pub(super) fn num_locals(&self) -> usize {
        self.locals
    }

    fn word_and_shift(&self, statement: usize, local: Local) -> (usize, u32) {
        assert!(statement < self.statements);
        assert!(local.index() < self.locals);

        let var = local.index();
        let word = var / Self::WORD_RWC_COUNT;
        let shift = Self::RWC_BITS * (var % Self::WORD_RWC_COUNT);
        (statement * self.statement_words + word, shift as u32)
    }

    pub(super) fn get_consume(&self, statement: usize, local: Local) -> bool {
        let (word, shift) = self.word_and_shift(statement, local);
        (self.words[word] >> shift) & Self::RWC_CONSUME != 0
    }

    pub(super) fn get_read(&self, statement: usize, local: Local) -> bool {
        let (word, shift) = self.word_and_shift(statement, local);
        (self.words[word] >> shift) & Self::RWC_READ != 0
    }

    pub(super) fn get_write(&self, statement: usize, local: Local) -> bool {
        let (word, shift) = self.word_and_shift(statement, local);
        (self.words[word] >> shift) & Self::RWC_WRITE != 0
    }

    pub(super) fn get_reads(&self, statement: usize) -> impl Iterator<Item = Local> + '_ {
        (0..self.locals)
            .map(Local::new)
            .filter(move |&local| self.get_read(statement, local))
    }

    pub(super) fn set_access(&mut self, statement: usize, local: Local, access: Access) {
        use Access::*;
        let (word, shift) = self.word_and_shift(statement, local);
        let word = &mut self.words[word];
        let mut rwc = (*word >> shift) & Self::RWC_MASK;
        match access {
            Read => rwc |= Self::RWC_READ,
            Write => rwc |= Self::RWC_WRITE,
            ReadWrite => rwc |= Self::RWC_READ | Self::RWC_WRITE,
            // if it is written, it is not considered consumed
            ReadConsume if rwc & Self::RWC_WRITE == 0 => rwc |= Self::RWC_READ | Self::RWC_CONSUME,
            ReadConsume => rwc |= Self::RWC_READ,
            NoAccess => {},
        }
        *word |= rwc << shift;
    }
}
