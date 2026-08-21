

// Marks a test function
#[derive(Clone)]
pub struct TestCase {
    f: fn(),
    name: &'static str,
    path: Location
}

impl Eq for TestCase {}

impl PartialEq<Self> for TestCase {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.location().eq(other.location()) && self.name.eq(other.name)
    }
}


impl Ord for TestCase {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.location().cmp(other.location()).then_with(|| self.name.cmp(other.name))
    }
}

impl PartialOrd<Self> for TestCase {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.location().cmp(other.location()).then_with(|| self.name.cmp(other.name) ))
    }
}

#[cfg(any(feature = "testing", feature = "harness"))]
inventory::collect!(&'static TestCase);

impl TestCase {

    pub const fn new(f: fn(), name: &'static str, path: Location) -> Self {
        Self { f, name, path }
    }


    /// Returns the test function
    #[inline(always)]
    pub fn function(&self) -> fn() { self.f }

    /// Returns the name of the function
    #[inline(always)]
    pub fn fn_name(&self) -> &'static str { self.name }

    /// Returns the path to the test function
    #[inline(always)]
    pub fn location(&self) -> &Location { &self.path }

}


/// Location of a test within a filesystem
#[derive(Clone)]
pub struct Location {
    file: &'static str,
    line: u32,
}

impl Eq for Location {}

impl PartialEq<Self> for Location {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.file().eq(other.file()) && self.line().eq(&other.line())
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        self.file().ne(other.file()) || self.line().ne(&other.line())
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.file().cmp(other.file()).then_with(|| self.line().cmp(&other.line()) )
    }
}

impl PartialOrd<Self> for Location {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.file().cmp(other.file()).then_with(|| self.line().cmp(&other.line()) ))
    }
}


impl Location {
    pub const fn new(file: &'static str, line: u32) -> Self {
        Self { file, line }
    }

    /// Returns the file where the test is declared
    #[inline(always)]
    pub fn file(&self) -> &'static str { self.file }

    /// Returns the line where the test is registered
    #[inline(always)]
    pub fn line(&self) -> u32 { self.line }
}

impl core::fmt::Debug for Location {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.file(), self.line())
    }
}
