use core::ops::{Add, AddAssign};
use core::default::Default;
use nonzero_ext::{NonZero, NonZeroAble};
use num_traits::{One, WrappingAdd, Zero};

/// A type which can be used as the index of a generation which may not be able to be incremented
pub trait FixedGenerationalIndex: Copy + Eq {
    /// Get an object representing the first possible generation
    fn first_generation() -> Self;
    /// Compare this generation with another.
    fn generation_lt(&self, other: &Self) -> bool;
}

/// A type which can be used as the index of a generation, which can be incremented
pub trait GenerationalIndex: FixedGenerationalIndex {
    /// Increment the generation of this object. May wrap or panic on overflow depending on type.
    fn increment_generation(&mut self);
}

/// A generation counter which is always nonzero. Useful for size optimizations on Option<Index>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NonzeroGeneration<T: NonZeroAble> {
    gen: T::NonZero,
}

impl<T> FixedGenerationalIndex for NonzeroGeneration<T>
where
    T: NonZeroAble
        + One
        + Add<Output = T>
        + Copy
        + Eq
        + From<<<T as NonZeroAble>::NonZero as NonZero>::Primitive>,
    T::NonZero: PartialOrd + Eq + Copy,
{
    #[inline(always)]
    fn first_generation() -> Self {
        NonzeroGeneration {
            gen: T::one().into_nonzero().unwrap(),
        }
    }
    #[inline(always)]
    fn generation_lt(&self, other: &Self) -> bool {
        self.gen < other.gen
    }
}

impl<T> GenerationalIndex for NonzeroGeneration<T>
where
    T: NonZeroAble
        + One
        + Add<Output = T>
        + Copy
        + Eq
        + From<<<T as NonZeroAble>::NonZero as NonZero>::Primitive>,
    T::NonZero: PartialOrd + Eq + Copy,
{
    #[inline(always)]
    fn increment_generation(&mut self) {
        self.gen = (T::from(self.gen.get()) + T::one()).into_nonzero().unwrap()
    }
}

/// A wrapping generation counter which is always nonzero.
/// Useful for size optimizations on Option<Index>
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NonzeroWrapGeneration<T: NonZeroAble> {
    gen: T::NonZero,
}

impl<T> FixedGenerationalIndex for NonzeroWrapGeneration<T>
where
    T: NonZeroAble
        + One
        + Zero
        + Copy
        + Eq
        + WrappingAdd
        + From<<<T as NonZeroAble>::NonZero as NonZero>::Primitive>,
    T::NonZero: PartialOrd + Eq + Copy,
{
    #[inline(always)]
    fn first_generation() -> Self {
        NonzeroWrapGeneration {
            gen: T::one().into_nonzero().unwrap(),
        }
    }
    #[inline(always)]
    fn generation_lt(&self, other: &Self) -> bool {
        self.gen < other.gen
    }
}

impl<T> GenerationalIndex for NonzeroWrapGeneration<T>
where
    T: NonZeroAble
        + One
        + Zero
        + Copy
        + Eq
        + WrappingAdd
        + From<<<T as NonZeroAble>::NonZero as NonZero>::Primitive>,
    T::NonZero: PartialOrd + Eq + Copy,
{
    #[inline(always)]
    fn increment_generation(&mut self) {
        let new = T::from(self.gen.get()).wrapping_add(&T::one());
        self.gen = if T::zero() == new {
            Self::first_generation().gen
        } else {
            new.into_nonzero().unwrap()
        }
    }
}

impl<T: Eq + One + AddAssign + Default + PartialOrd + Copy> FixedGenerationalIndex for T {
    #[inline(always)]
    fn first_generation() -> Self {
        Default::default()
    }
    #[inline(always)]
    fn generation_lt(&self, other: &Self) -> bool {
        self.lt(other)
    }
}

impl<T: Eq + One + AddAssign + Default + PartialOrd + Copy> GenerationalIndex for T {
    #[inline(always)]
    fn increment_generation(&mut self) {
        *self += Self::one()
    }
}

/// If this is used as a generational index, then the arena ignores generation
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IgnoreGeneration;

impl FixedGenerationalIndex for IgnoreGeneration {
    #[inline(always)]
    fn first_generation() -> Self {
        IgnoreGeneration
    }
    #[inline(always)]
    fn generation_lt(&self, _other: &Self) -> bool {
        false
    }
}

impl GenerationalIndex for IgnoreGeneration {
    #[inline(always)]
    fn increment_generation(&mut self) {}
}

/// A marker trait which says that a generation type is ignored
pub trait IgnoredGeneration: FixedGenerationalIndex {}
impl IgnoredGeneration for IgnoreGeneration {}

/// If this is used as a generational index, then the arena is no longer generational
/// and does not allow element removal
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DisableRemoval;

impl FixedGenerationalIndex for DisableRemoval {
    #[inline(always)]
    fn first_generation() -> Self {
        DisableRemoval
    }
    #[inline(always)]
    fn generation_lt(&self, _other: &Self) -> bool {
        false
    }
}

impl IgnoredGeneration for DisableRemoval {}

