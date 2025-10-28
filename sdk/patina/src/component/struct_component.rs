//! A [Component] implementation for Structs who specify a function whose parameters implement [Param].
//!
//! The `StructComponent` is a component that allows for private internal configuration and requires the use of an
//! derive proc-macro to be used on the struct or enum to implement necessary traits and specify the entry point
//! function for the component.
//!
//! A derive macro, [IntoComponent](crate::component::IntoComponent) is provided to automatically implement the
//! necessary traits for a struct or enum to be used as a component. This trait expects that a default entry point
//! function of `Self::entry_point` exists. This can be overridden with the `#[entry_point(path = path::to::function)]`
//! attribute.
//!
//! It is important to note that the function's first parameter must be `self` or `mut self`, **NOT** `&self` or
//! `&mut self`. This design choice was made as components are only expected to be executed once, and by consuming
//! `self`, you are able to pass ownership of the entire struct (or items within the struct) to other "things" (for
//! lack of a better term) without the need for cloning or borrowing.
//!
//! Review [Param] implementations for all types that can be used as parameters to these functions.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
extern crate alloc;

use crate::{
    component::{
        Component,
        metadata::MetaData,
        params::{ComponentInput, Param, ParamFunction},
        storage::{Storage, UnsafeStorageCell},
    },
    error::Result,
};
use core::marker::PhantomData;

/// A [Component] implementation for Structs who specify a function whose parameters implement [Param].
pub struct StructComponent<Marker, Func>
where
    Func: ParamFunction<Marker>,
{
    func: Func,
    input: Option<Func::In>,
    param_state: Option<<Func::Param as Param>::State>,
    metadata: MetaData,
    _marker: PhantomData<fn() -> Marker>,
}

impl<Marker, Func> StructComponent<Marker, Func>
where
    Marker: 'static,
    Func: ParamFunction<Marker>,
{
    /// Creates a new `struct` component with the given function and input.
    pub fn new(func: Func, input: Func::In) -> Self {
        Self {
            func,
            input: Some(input),
            param_state: None,
            metadata: MetaData::new::<Func::In>(),
            _marker: PhantomData,
        }
    }
}

impl<Marker, In, Func> Component for StructComponent<Marker, Func>
where
    Marker: 'static,
    In: ComponentInput + 'static,
    Func: ParamFunction<Marker, In = In, Out = Result<()>>,
{
    /// Runs the Component if all parameters are retrievable from storage.
    ///
    /// ## Safety
    ///
    /// - Each parameter must properly register its access type.
    /// - Each parameter must properly validate its access ability.
    unsafe fn run_unsafe(&mut self, storage: UnsafeStorageCell) -> Result<bool> {
        let param_state = self.param_state.as_mut().expect("Param state created on initialize.");

        if let Err(bad_param) = Func::Param::try_validate(param_state, storage) {
            self.metadata.set_failed_param(bad_param);
            return Ok(false);
        }

        let param_value = unsafe { Func::Param::get_param(param_state, storage) };

        debug_assert!(
            self.input.is_some(),
            "{} `input` is `None` during run. Did this component already run?",
            core::any::type_name::<Self>()
        );
        self.func.run(&mut self.input, param_value).map(|_| true)
    }

    /// Returns the metadata of the Component.
    fn metadata(&self) -> &MetaData {
        &self.metadata
    }

    /// One-time initialization of the Component. Should set [Access](super::metadata::Access) requirements.
    fn initialize(&mut self, _storage: &mut Storage) {
        self.param_state = Some(Func::Param::init_state(_storage, &mut self.metadata));
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use crate as patina;
    use crate::component::{
        IntoComponent,
        params::{Config, ConfigMut},
    };

    #[derive(IntoComponent)]
    #[entry_point(path = TestStructSuccess::entry_point)]
    #[allow(dead_code)]
    pub struct TestStructSuccess {
        pub x: i32,
    }

    impl TestStructSuccess {
        fn entry_point(self, _cfg: crate::component::params::Config<i32>) -> crate::error::Result<()> {
            Ok(())
        }
    }

    #[derive(IntoComponent)]
    #[entry_point(path = enum_entry_point)]
    #[allow(dead_code)]
    pub enum TestEnumSuccess {
        A,
        B,
    }

    fn enum_entry_point(_s: TestEnumSuccess, _cfg: Config<i32>) -> crate::error::Result<()> {
        Ok(())
    }

    #[derive(crate::component::IntoComponent)]
    #[allow(dead_code)]
    pub struct TestStructNotDispatched {
        pub x: i32,
    }

    impl TestStructNotDispatched {
        fn entry_point(self, _cfg: ConfigMut<u32>) -> crate::error::Result<()> {
            Ok(())
        }
    }

    #[derive(crate::component::IntoComponent)]
    #[allow(dead_code)]
    pub struct TestStructFail {
        pub x: i32,
    }

    impl TestStructFail {
        fn entry_point(self) -> crate::error::Result<()> {
            Err(crate::error::EfiError::NotReady)
        }
    }

    #[test]
    fn test_struct_component() {
        let test_struct = TestStructSuccess { x: 5 };
        let _ = test_struct.into_component();
    }

    #[test]
    fn test_enum_component() {
        let test_enum = TestEnumSuccess::A;
        let _ = test_enum.into_component();
    }

    #[test]
    fn test_component_run_handling_works_as_expected() {
        let mut storage = crate::component::storage::Storage::new();

        let mut test_struct = TestStructSuccess { x: 5 }.into_component();
        test_struct.initialize(&mut storage);
        assert!(test_struct.run(&mut storage).is_ok_and(|res| res));

        let mut test_enum = TestEnumSuccess::A.into_component();
        test_enum.initialize(&mut storage);
        assert!(test_enum.run(&mut storage).is_ok_and(|res| res));

        let mut test_struct = TestStructNotDispatched { x: 5 }.into_component();
        test_struct.initialize(&mut storage);
        storage.lock_configs(); // Lock it so the ConfigMut can't be accessed
        assert!(test_struct.run(&mut storage).is_ok_and(|res| !res));
        assert_eq!(test_struct.metadata().failed_param(), Some("patina::component::params::ConfigMut<'_, u32>"));

        let mut test_struct = TestStructFail { x: 5 }.into_component();
        test_struct.initialize(&mut storage);
        assert!(test_struct.run(&mut storage).is_err_and(|res| res == crate::error::EfiError::NotReady));
    }

    //Test structs that use generics and where clause
    #[derive(crate::component::IntoComponent)]
    struct GenericStruct<T>
    where
        T: 'static,
    {
        _x: T,
    }

    impl<T> GenericStruct<T> {
        fn entry_point(self, _cfg: Config<u32>) -> crate::error::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_generic_struct_can_be_component() {
        let test_struct = GenericStruct { _x: 5 };
        let _ = test_struct.into_component();
    }

    #[derive(crate::component::IntoComponent)]
    struct GenericStruct2<T: 'static> {
        _x: T,
    }

    impl<T: 'static> GenericStruct2<T> {
        fn entry_point(self, _cfg: Config<u32>) -> crate::error::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_generic_struct_with_where_clause_can_be_component() {
        let test_struct = GenericStruct2 { _x: 5 };
        let _ = test_struct.into_component();
    }

    #[test]
    /// A test that will stop compiling if we lose the ability to take self by value (self).
    fn test_component_entry_point_that_take_by_value_works() {
        #[derive(crate::component::IntoComponent)]
        struct ByValue {
            _x: u32,
        }

        impl ByValue {
            fn entry_point(self, _cfg: Config<u32>) -> crate::error::Result<()> {
                Ok(())
            }
        }

        let _ = ByValue { _x: 5 }.into_component();
    }

    #[test]
    /// A test that will stop compiling if we lose the ability to take self by ref (&self).
    fn test_component_entry_point_that_take_by_ref_works() {
        #[derive(crate::component::IntoComponent)]
        struct ByRef {
            _x: u32,
        }

        impl ByRef {
            fn entry_point(&self, _cfg: Config<u32>) -> crate::error::Result<()> {
                Ok(())
            }
        }

        let _ = ByRef { _x: 5 }.into_component();
    }

    #[test]
    /// A test that will stop compiling if we lose the ability to take self by ref (&mut self).
    fn test_component_entry_point_that_take_by_mut_works() {
        #[derive(crate::component::IntoComponent)]
        struct ByMut {
            _x: u32,
        }

        impl ByMut {
            fn entry_point(&mut self, _cfg: Config<u32>) -> crate::error::Result<()> {
                Ok(())
            }
        }

        let _ = ByMut { _x: 5 }.into_component();
    }
}
