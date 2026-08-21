
//Traits

#[macro_export]
macro_rules! trait_get
{

    ($name:ident, $name_type:ty) =>
    {

        fn $name(&self) -> $name_type;

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        #[doc = $documentation]
        fn $name(&self) -> $name_type;

    }

}

#[macro_export]
macro_rules! trait_set
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<set_ $name>](&mut self, value: $name_type);

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<set_ $name>](&mut self, value: $name_type);

        }

    }

}

#[macro_export]
macro_rules! trait_get_set
{

    ($name:ident, $name_type:ty) =>
    {

        trait_get!($name, $name_type);

        trait_set!($name, $name_type);

    };
    ($name:ident, $name_type:ty, $getter_documentation:literal, $setter_documentation:literal) =>
    {

        trait_get!($name, $name_type, $getter_documentation);

        trait_set!($name, $name_type, $setter_documentation);

    }

}

#[macro_export]
macro_rules! trait_get_ref
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<$name _ref>](&self) -> &$name_type;

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<$name _ref>](&self) -> &$name_type;
            
        }

    }

}

#[macro_export]
macro_rules! trait_get_mut
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<$name _mut>](&mut self) -> &mut $name_type;

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<$name _mut>](&mut self) -> &mut $name_type;

        }

    }

}

//Cloning

//Getter

#[macro_export]
macro_rules! trait_get_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn $name(&self) -> $name_type;

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn $name(&self) -> $name_type;
            
        }

    }

}

//Setter

/*
#[macro_export]
macro_rules! trait_set_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<set_ $name _clone>](&mut self, value: &$name_type);

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<set_ $name _clone>](&mut self, value: &$name_type);

        }

    }

}
*/

//Impl Blocks

#[macro_export]
macro_rules! impl_get
{

    ($name:ident, $name_type:ty) =>
    {

        pub fn $name(&self) -> $name_type
        {

            self.$name

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        #[doc = $documentation]
        pub fn $name(&self) -> $name_type
        {

            self.$name

        }

    }

}

//Impl Trait

#[macro_export]
macro_rules! impl_trait_get
{

    ($name:ident, $name_type:ty) =>
    {

        fn $name(&self) -> $name_type
        {

            self.$name

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        #[doc = $documentation]
        fn $name(&self) -> $name_type
        {

            self.$name

        }

    }

}

//Set

#[macro_export]
macro_rules! impl_set
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn [<set_ $name>](&mut self, value: $name_type)
            {

                self.$name = value;

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            pub fn [<set_ $name>](&mut self, value: $name_type)
            {

                self.$name = value;

            }

        }

    }

}

//Impl Trait

#[macro_export]
macro_rules! impl_trait_set
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<set_ $name>](&mut self, value: $name_type)
            {

                self.$name = value;

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<set_ $name>](&mut self, value: $name_type)
            {

                self.$name = value;

            }

        }

    }

}

#[macro_export]
macro_rules! impl_get_set
{

    ($name:ident, $name_type:ty) =>
    {

        impl_get!($name, $name_type);

        impl_set!($name, $name_type);

    };
    ($name:ident, $name_type:ty, $getter_documentation:literal, $setter_documentation:literal) =>
    {

        impl_get!($name, $name_type, $getter_documentation);

        impl_set!($name, $name_type, $setter_documentation:);

    }

}

//References

//Ref

#[macro_export]
macro_rules! impl_get_ref
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn [<$name _ref>](&self) -> &$name_type
            {

                &self.$name

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            pub fn [<$name _ref>](&self) -> &$name_type
            {

                &self.$name

            }

        }

    }

}

//Impl Trait

#[macro_export]
macro_rules! impl_trait_get_ref
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<$name _ref>](&self) -> &$name_type
            {

                &self.$name

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<$name _ref>](&self) -> &$name_type
            {

                &self.$name

            }

        }

    }

}

//Mut

#[macro_export]
macro_rules! impl_get_mut
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn [<$name _mut>](&mut self) -> &mut $name_type
            {

                &mut self.$name

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            pub fn [<$name _mut>](&mut self) -> &mut $name_type
            {

                &mut self.$name

            }

        }

    }

}

//Impl Trait

//Getter

#[macro_export]
macro_rules! impl_trait_get_mut
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<$name _mut>](&mut self) -> &mut $name_type
            {

                &mut self.$name

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<$name _mut>](&mut self) -> &mut $name_type
            {

                &mut self.$name

            }

        }

    }

}

//Cloning

#[macro_export]
macro_rules! impl_get_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn $name(&self) -> $name_type
            {

                self.$name.clone()

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            pub fn $name(&self) -> $name_type
            {

                self.$name.clone()

            }
            
        }

    }

}

//Impl Trait

#[macro_export]
macro_rules! impl_trait_get_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn $name(&self) -> $name_type
            {

                self.$name.clone()

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn $name(&self) -> $name_type
            {

                self.$name.clone()

            }
            
        }

    }

}

//Setter

#[macro_export]
macro_rules! impl_set_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn [<set_ $name>](&mut self, value: &$name_type)
            {

                self.$name = value.clone();

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            pub fn [<set_ $name>](&mut self, value: &$name_type)
            {

                self.$name = value.clone();

            }

        }

    }

}

//Impl Trait

#[macro_export]
macro_rules! impl_trait_set_clone
{

    ($name:ident, $name_type:ty) =>
    {

        paste!
        {

            fn [<set_ $name>](&mut self, value: &$name_type)
            {

                self.$name = value.clone();

            }

        }

    };
    ($name:ident, $name_type:ty, $documentation:literal) =>
    {

        paste!
        {

            #[doc = $documentation]
            fn [<set_ $name>](&mut self, value: &$name_type)
            {

                self.$name = value.clone();

            }

        }

    }

}

/*
#[macro_export]
macro_rules! impl_aliased_get_ref
{

    ($field:ident, $alias:ident, $name_type:ty) =>
    {

        pub fn $alias(&self) -> &$name_type
        {

            &self.$field

        }

    }

}

#[macro_export]
macro_rules! impl_aliased_get_mut
{

    ($field:ident, $alias:ident, $name_type:ty) =>
    {

        paste!
        {

            pub fn $alias(&mut self) -> &mut $name_type
            {

                &mut self.$field

            }

        }

    }

}
*/






