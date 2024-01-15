use std::collections::BTreeSet;

mod utils {
    pub fn is_valid_library_component(str: &String) -> bool {
        true
    }
}

/// A platform represents a group of FIDL libraries that are versioned together.
/// Usually all the library names begin with a common prefix, the platform name.
/// Libraries that don't use versioning belong to an anonymous platform.
#[derive(Debug, PartialEq, Eq)]
pub struct Platform(Option<String>);

impl Platform {
    /// Creates an anonymous platform.
    fn anonymous() -> Self {
        return Platform(None);
    }

    /// Creates a named platform. Returns null if `str` is not a valid name.
    pub(crate) fn parse(str: &String) -> Option<Self> {
        if utils::is_valid_library_component(str) {
            return Some(Platform(Some(str.clone())));
        }

        None
    }

    /// Returns true if this is an anonymous platform.
    fn is_anonymous(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the platform's name. Panics if this is an anonymous platform.
    fn name(&self) -> &String {
        assert!(
            self.0.is_some(),
            "Platform::name() must not be called on anonymous platforms"
        );
        &self.0.unwrap()
    }
}

/// A version represents a particular state of a platform.
///
/// Versions are categorized like so:
///
///     Finite
///         Numeric -- 1, 2, ..., 2^63-1
///         HEAD    -- the unstable, most up-to-date version
///         LEGACY  -- HEAD plus legacy elements
///     Infinite
///         -inf    -- the infinite past
///         +inf    -- the infinite future
///
/// Infinite versions help avoid special cases in algorithms. For example, in a
/// FIDL library that has no @available attributes at all, everything is
/// considered added at HEAD and removed at +inf.
///
/// A finite version's ordinal is the uint64 format specified in RFC-0083:
///
///               { numeric versions }                       HEAD  LEGACY
///        o------o------o--- ... ---o------o--- ... ---o------o------o
///        0      1      2        2^63-1   2^63     2^64-3  2^64-2  2^64-1
///
/// Internally, this class uses a different format to represent -inf and +inf:
///
///      -inf     { numeric versions }                HEAD  LEGACY  +inf
///        o------o------o--- ... ---o------o--- ... ---o------o------o
///        0      1      2        2^63-1   2^63     2^64-2   2^64-1
///
/// NOTE: that HEAD and LEGACY are bumped down to make comparisons work properly.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64);

impl Version {
    fn from(ordinal: u64) -> Option<Version> {
        None
    }

    /// Succeeds if `str` can be parsed as a numeric version, or is "HEAD" or "LEGACY".
    fn parse(str: String) -> Option<Version> {
        None
    }

    /// Special version before all others. "Added at -inf" means "no beginning".
    fn neg_inf() -> Self {
        Version(0)
    }

    /// Special version after all others. "Removed at +inf" means "no end".
    fn pos_inf() -> Self {
        Version(u64::MAX)
    }

    /// Special version meaning "the unstable, most up-to-date version".
    fn head() -> Self {
        Version(u64::MAX - 2)
    }

    /// Special version that is like HEAD but includes legacy elements.
    fn legacy() -> Self {
        Version(u64::MAX - 1)
    }

    /// Returns the version's ordinal. Assumes the version is finite.
    fn ordinal(&self) -> u64 {
        self.0
    }

    /// Returns a string representation of the version.
    fn to_string(&self) -> String {
        const NEG_INF: u64 = Version::neg_inf().ordinal();
        const POS_INF: u64 = Version::pos_inf().ordinal();
        const HEAD: u64 = Version::head().ordinal();
        const LEGACY: u64 = Version::legacy().ordinal();

        match self.0 {
            NEG_INF => String::from("-inf"),
            POS_INF => String::from("+inf"),
            HEAD => String::from("HEAD"),
            LEGACY => String::from("LEGACY"),
            _ => self.0.to_string(),
        }
    }
}

struct Legacy {}

/// An availability advances through four states. All reach kNarrowed on
/// success, except for library availabilities, which stay at kInherited
/// because libraries do not get decomposed.
#[derive(Debug, PartialEq)]
enum AvailabilityState {
    // 1. Default constructed. All fields are null.
    Unset,
    // 2. `Init` succeeded. Some fields might be set, and they are in order.
    Initialized,
    // 3. `Inherit` succeeded. Now `added`, `removed`, and `legacy` are always set.
    Inherited,
    // 4. `Narrow` succeeded. Now `deprecated` is unset or equal to `added`, and
    //    `legacy` is either kNotApplicable or kNo.
    Narrowed,
    // One of the steps failed.
    Failed,
}

/// Represents whether an availability includes legacy support.
#[derive(Debug, PartialEq)]
enum AvailabilityLegacy {
    // Not applicable because [added, removed) already includes LEGACY,
    // i.e. `removed` is +inf.
    NotApplicable,
    // No legacy support: do not re-add at LEGACY.
    No,
    // Legacy support: re-add at LEGACY.
    Yes,
}

/// An availability represents the versions when a FIDL element was added (A),
/// deprecated (D), removed (R), and re-added as legacy (L) in a platform. These
/// versions break the platform's timeline into the following regions:
///
///     Present        -- [A, R) and [L, +inf) if L is set
///         Available  -- [A, D or R)
///         Deprecated -- [D, R) if D is set
///         Legacy     -- [L, +inf) if L is set
///     Absent         -- (-inf, A) and [R, L or +inf)
///
/// Here is what the timeline looks like for finite versions A, D, R:
///
///   -inf         A-1  A              D-1  D              R-1  R         +inf
///     o--- ... ---o---o--- ....... ---o---o--- ....... ---o---o--- ... ---o
///     |           |   |-- Available --|   |-- Deprecated -|   |           |
///     |-- Absent -|   |-------------- Present ------------|   |-- Absent -|
///
/// Here is what the timeline looks like for a legacy element (L = LEGACY):
///
///   -inf         A-1  A              R-1  R          L-1   L          +inf
///     o--- ... ---o---o--- ....... ---o---o--- ... ---o----o---- ... ---o
///     |           |   |-- Available --|   |           |    |-- Legacy --|
///     |-- Absent -|   |--- Present ---|   |-- Absent -|    |-- Present -|
///
/// Here is what the timeline looks like for Availability::Unbounded():
///
///   -inf                                                                +inf
///     o-------------------------------------------------------------------o
///     |---------------------------- Available ----------------------------|
///     |----------------------------- Present -----------------------------|
///
pub struct Availability {
    state: AvailabilityState,

    added: Option<Version>,
    deprecated: Option<Version>,
    removed: Option<Version>,

    legacy: Option<AvailabilityLegacy>,
}

#[derive(Debug, PartialEq, Eq)]
enum InheritResultStatus {
    Ok,
    // Child {added, deprecated, or removed} < Parent added.
    BeforeParentAdded,
    // Child deprecated > Parent deprecated.
    AfterParentDeprecated,
    // Child {added or deprecated} >= Parent removed,
    // or Child removed > Parent removed.
    AfterParentRemoved,
}

#[derive(Debug, PartialEq, Eq)]
enum InheritResultLegacyStatus {
    Ok,
    // Child marked `legacy=false` or `legacy=true`, but was never removed
    // (neither directly nor through inheritance from parent).
    NeverRemoved,
    // Child legacy is kYes but Parent legacy is kNo, and both are removed.
    WithoutParent,
}

struct InheritResult {
    added: InheritResultStatus,
    deprecated: InheritResultStatus,
    removed: InheritResultStatus,
    legacy: InheritResultLegacyStatus,
}

impl InheritResult {
    fn new() -> Self {
        Self {
            added: InheritResultStatus::Ok,
            deprecated: InheritResultStatus::Ok,
            removed: InheritResultStatus::Ok,
            legacy: InheritResultLegacyStatus::Ok,
        }
    }

    fn ok(&self) -> bool {
        return self.added == InheritResultStatus::Ok
            && self.deprecated == InheritResultStatus::Ok
            && self.removed == InheritResultStatus::Ok
            && self.legacy == InheritResultLegacyStatus::Ok;
    }
}

impl Availability {
    fn state(&self) -> AvailabilityState {
        self.state
    }

    /// Returns the points demarcating the availability: `added`, `removed`,
    /// `deprecated` (if deprecated), and LEGACY and +inf (if Legacy::Yes).
    /// Must be in the Inherited or kNarrowed state.
    fn points(&self) -> BTreeSet<Version> {
        assert!(self.state == AvailabilityState::Inherited || self.state == AvailabilityState::Narrowed);

        let mut result = BTreeSet::new();
        result.insert(self.added.unwrap());
        result.insert(self.removed.unwrap());

        if self.deprecated.is_some() {
            result.insert(self.deprecated.unwrap());
        }

        if self.legacy.unwrap() == AvailabilityLegacy::Yes {
            assert!(result.insert(Version::legacy()));
            assert!(result.insert(Version::pos_inf()));
        }

        result
    }

    /// Returns true if the whole range is deprecated, and false if none of it is.
    /// Must be in the kNarrowed state (where deprecation is all-or-nothing).
    fn is_deprecated(&self) -> bool {
        assert!(self.state == AvailabilityState::Narrowed);
        self.deprecated.is_some()
    }

    /// Explicitly mark the availability as failed. Must not have called Init yet.
    fn fail(&mut self) {
        assert!(self.state == AvailabilityState::Unset, "called Fail in the wrong order");
        self.state = AvailabilityState::Failed;
    }

    /// Must be called second. Inherits unset fields from `parent`.
    fn inherit(&self, parent: &Availability) -> InheritResult {
        assert!(
            self.state == AvailabilityState::Initialized,
            "called Inherit in the wrong order"
        );
        assert!(
            parent.state == AvailabilityState::Inherited,
            "must call Inherit on parent first"
        );

        let mut result = InheritResult::new();

        // Inherit and validate `added`.
        if self.added.is_none() {
            self.added = parent.added;
        } else if self.added.unwrap() < parent.added.unwrap() {
            result.added = InheritResultStatus::BeforeParentAdded;
        } else if self.added.unwrap() >= parent.removed.unwrap() {
            result.added = InheritResultStatus::AfterParentRemoved;
        }

        // Inherit and validate `removed`.
        if self.removed.is_none() {
            self.removed = parent.removed;
        } else if self.removed.unwrap() <= parent.added.unwrap() {
            result.removed = InheritResultStatus::BeforeParentAdded;
        } else if self.removed.unwrap() > parent.removed.unwrap() {
            result.removed = InheritResultStatus::AfterParentRemoved;
        }

        // Inherit and validate `deprecated`.
        if self.deprecated.is_none() {
            // Only inherit deprecation if it occurs before this element is removed.
            if parent.deprecated.is_some() && parent.deprecated.unwrap() < self.removed.unwrap() {
                // As a result of inheritance, we can end up with deprecated < added:
                //
                //     @available(added=1, deprecated=5, removed=10)
                //     type Foo = struct {
                //         @available(added=7)
                //         bar bool;
                //     };
                //
                // To maintain `added <= deprecated < removed` in this case, we use
                // Version::max below. A different choice would be to disallow this, and
                // consider `Foo` frozen once deprecated. However, going down this path
                // leads to contradictions with the overall design of FIDL Versioning.
                self.deprecated = Some(Version::max(parent.deprecated.unwrap(), self.added.unwrap()));
            }
        } else if self.deprecated.unwrap() < parent.added.unwrap() {
            result.deprecated = InheritResultStatus::BeforeParentAdded;
        } else if self.deprecated.unwrap() >= parent.removed.unwrap() {
            result.deprecated = InheritResultStatus::AfterParentRemoved;
        } else if parent.deprecated.is_some() && self.deprecated.unwrap() > parent.deprecated.unwrap() {
            result.deprecated = InheritResultStatus::AfterParentDeprecated;
        }

        // Inherit and validate `legacy`.
        if self.legacy.is_none() {
            if self.removed.unwrap() == parent.removed.unwrap() {
                // Only inherit if the parent was removed at the same time. For example:
                //
                //     @available(added=1, removed=100, legacy=true)
                //     type Foo = table {
                //         @available(removed=2) 1: string bar;
                //         @available(added=2)   1: string bar:10;
                //         @available(removed=3) 2: bool qux;
                //     };
                //
                // It's crucial we do not inherit legacy=true on the first `bar`,
                // otherwise there will be two `bar` fields that collide at LEGACY. We
                // also don't want to inherit legacy=true for `qux`: it had no legacy
                // legacy support when it was removed at 3, so it doesn't make sense to
                // change that when we later remove the entire table at 100.
                //
                // An alternative is to inherit when the child has no explicit `removed`.
                // We prefer to base it on post-inheritance equality so that adding or
                // removing a redundant `removed=...` on the child is purely stylistic.
                self.legacy = parent.legacy;
            } else {
                assert!(
                    self.removed.unwrap() != Version::pos_inf(),
                    "impossible for child to be removed at +inf if parent is not also removed at +inf"
                );
                // By default, removed elements are not added back at LEGACY.
                self.legacy = Some(AvailabilityLegacy::No);
            }
        } else if self.removed.unwrap() == Version::pos_inf() {
            // Legacy is not applicable if the element is never removed. Note that we
            // cannot check this earlier (e.g. in Init) because we don't know if the
            // element is removed or not until performing inheritance.
            result.legacy = InheritResultLegacyStatus::NeverRemoved;
        } else if self.legacy.unwrap() == AvailabilityLegacy::Yes && parent.legacy.unwrap() == AvailabilityLegacy::No {
            // We can't re-add the child at LEGACY without its parent.
            result.legacy = InheritResultLegacyStatus::WithoutParent;
        }

        if result.ok() {
            assert!(self.added.is_some() && self.removed.is_some() && self.legacy.is_some());
            assert!(self.added.unwrap() != Version::neg_inf());
            assert!(self.valid_order());
            self.state = AvailabilityState::Inherited;
        } else {
            self.state = AvailabilityState::Failed;
        }

        result
    }

    fn valid_order(&self) -> bool {
        let a = self.added.unwrap_or(Version::neg_inf());
        let d = self.deprecated.unwrap_or(a);
        let r = self.removed.unwrap_or(Version::pos_inf());
        return a <= d && d < r;
    }
}
