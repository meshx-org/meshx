#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub struct Platform;

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
pub struct Version(u64);

impl Version {
    /// Special version before all others. "Added at -inf" means "no beginning".
    pub fn neg_inf() -> Self {
        Version(0)
    }

    /// Special version after all others. "Removed at +inf" means "no end".
    pub fn pos_inf() -> Self {
        Version(u64::MAX)
    }

    /// Special version meaning "the unstable, most up-to-date version".
    pub fn head() -> Self {
        Version(u64::MAX - 2)
    }

    /// Special version that is like HEAD but includes legacy elements.
    pub fn legacy() -> Self {
        Version(u64::MAX - 1)
    }
}

#[derive(PartialEq, Debug)]
pub enum Legacy {
    NotApplicable,
    No,
}

/// An availability advances through four states. All reach Narrowed on
/// success, except for library availabilities, which stay at Inherited
/// because libraries do not get decomposed.
#[derive(PartialEq, Debug, Default)]
enum AvailabilityState {
    /// 1. Default constructed. All fields are null.
    #[default]
    Unset,
    /// 2. `Init` succeeded. Some fields might be set, and they are in order.
    Initialized,
    /// 3. `Inherit` succeeded. Now `added`, `removed`, and `legacy` are always set.
    Inherited,
    /// 4. `Narrow` succeeded. Now `deprecated` is unset or equal to `added`, and
    ///    `legacy` is either NotApplicable or No.
    Narrowed,
    /// One of the steps failed.
    Failed,
}

#[derive(Debug, Default)]
pub struct Availability {
    state: AvailabilityState,
    added: Option<Version>,
    deprecated: Option<Version>,
    removed: Option<Version>,
    legacy: Option<Legacy>,
}

pub struct AvailabilityInitArgs {
    pub added: Option<Version>,
    pub deprecated: Option<Version>,
    pub removed: Option<Version>,
    pub legacy: Option<Legacy>,
}

type InheritResult = bool;

impl Availability {
    // Returns an availability that exists forever. This only exists as the base
    // case for calling `Inherit`. It never occurs as a final result.
    pub fn unbounded() -> Self {
        Self {
            state: AvailabilityState::Inherited,
            added: Some(Version::neg_inf()),
            removed: Some(Version::pos_inf()),
            deprecated: None,
            legacy: Some(Legacy::NotApplicable),
        }
    }

    /// Returns the presence range: [added, removed). Must be in the kNarrowed state.
    pub fn range(&self) -> VersionRange {
        assert!(self.state == AvailabilityState::Narrowed);
        VersionRange::new(self.added.unwrap(), self.removed.unwrap())
    }

    /// Must be called first. Initializes the availability from @available fields.
    /// Returns false if they do not satisfy `added <= deprecated < removed`.
    pub fn init(&mut self, args: AvailabilityInitArgs) -> bool {
        assert!(self.state == AvailabilityState::Unset, "called Init in the wrong order");
        assert!(
            self.legacy != Some(Legacy::NotApplicable),
            "legacy cannot be kNotApplicable"
        );

        for version in vec![args.added, args.deprecated, args.removed] {
            assert!(version != Some(Version::neg_inf()));
            assert!(version != Some(Version::pos_inf()));
            //    assert!(version != Version::legacy());
        }

        self.added = args.added;
        self.deprecated = args.deprecated;
        self.removed = args.removed;
        self.legacy = args.legacy;

        let valid = self.valid_order();

        self.state = if valid {
            AvailabilityState::Initialized
        } else {
            AvailabilityState::Failed
        };

        valid
    }

    /// Must be called second. Inherits unset fields from `parent`.
    pub fn inherit(&mut self, parent: &Availability) -> InheritResult {
        assert!(
            self.state == AvailabilityState::Initialized,
            "called Inherit in the wrong order"
        );
        assert!(
            parent.state == AvailabilityState::Inherited,
            "must call Inherit on parent first"
        );
        let result: InheritResult = false;

        result
    }

    /// Must be called third. Narrows the availability to the given range, which
    /// must be a subset of range()
    pub fn narrow(&mut self, range: VersionRange) {
        assert!(
            self.state == AvailabilityState::Inherited,
            "called Narrow in the wrong order"
        );
        let (a, b) = range.pair;

        if a == Version::legacy() {
            assert!(b == Version::pos_inf(), "legacy range must be [LEGACY, +inf)");
            assert!(self.legacy != Some(Legacy::No), "must be present at LEGACY");
        } else {
            assert!(
                a >= self.added.unwrap() && b <= self.removed.unwrap(),
                "must narrow to a subrange"
            );
        }

        self.added = Some(a);
        self.removed = Some(b);

        if self.deprecated.is_some() && a >= self.deprecated.unwrap() {
            self.deprecated = Some(a);
        } else {
            self.deprecated = None;
        }

        if a <= Version::legacy() && b > Version::legacy() {
            self.legacy = Some(Legacy::NotApplicable);
        } else {
            self.legacy = Some(Legacy::No);
        }

        self.state = AvailabilityState::Narrowed;
    }

    fn valid_order(&self) -> bool {
        let a = self.added.unwrap_or(Version::neg_inf());
        let d = self.deprecated.unwrap_or(a);
        let r = self.removed.unwrap_or(Version::pos_inf());

        return a <= d && d < r;
    }
}

pub struct VersionSelection;

impl VersionSelection {
    pub fn lookup(&self, platform: Platform) -> Version {
        // if platform.is_anonymous() {
        //     return Version::head();
        // }

        // let iter = self.map.find(platform);
        // assert!(iter != map_.end(), "no version was inserted for platform '%s'", platform.name().c_str());

        //return iter.second;

        Version::head()
    }
}

/// A version range is a nonempty set of versions in some platform, from an
/// inclusive lower bound to an exclusive upper bound.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct VersionRange {
    pair: (Version, Version),
}

impl VersionRange {
    pub fn new(lower: Version, upper_exclusive: Version) -> Self {
        assert!(lower < upper_exclusive, "invalid version range");

        Self {
            pair: (lower, upper_exclusive),
        }
    }

    /// Returns true if this range contains `version`.
    pub fn contains(&self, version: Version) -> bool {
        let (a, b) = self.pair;
        a <= version && version < b
    }

    /// Returns the intersection of two (possibly empty) ranges.
    pub fn intersect(lhs: Option<VersionRange>, rhs: Option<VersionRange>) -> Option<VersionRange> {
        if lhs.is_none() || rhs.is_none() {
            return None;
        }

        let (a1, b1) = lhs.unwrap().pair;
        let (a2, b2) = rhs.unwrap().pair;

        if b1 <= a2 || b2 <= a1 {
            return None;
        }

        Some(VersionRange::new(std::cmp::max(a1, a2), std::cmp::min(b1, b2)))
    }
}
