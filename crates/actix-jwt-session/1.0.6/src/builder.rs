use actix_web::dev::{self, ServiceFactory, ServiceRequest};
use actix_web::web::Data;
use actix_web::{App, Error as ActixWebError};
use deadpool_redis::Pool;

use crate::{Claims, Extractors, SessionMiddlewareFactory};

/**
    This trait gives the ability to call [`Self::use_jwt`] on the implemented type.
*/
pub trait UseJwt {
    /**
        Calls `wrap` on the `scope` will passing the `authority`.
        Then it adds the `scope` as a service on `self`.

        If there is a [`crate::TokenSigner`] set on the `authority`, it is clone it and adds it as app data on `self`.
    */
    fn use_jwt<AppClaims: Claims>(
        self,
        extractors: Extractors<AppClaims>,
        pool: Option<Pool>,
    ) -> App<
        impl ServiceFactory<
            dev::ServiceRequest,
            Error = actix_web::Error,
            Config = (),
            InitError = (),
            Response = dev::ServiceResponse,
        >,
    >;
}

impl<T> UseJwt for App<T>
where
    T: ServiceFactory<
            ServiceRequest,
            Config = (),
            Error = ActixWebError,
            InitError = (),
            Response = dev::ServiceResponse,
        > + 'static,
{
    fn use_jwt<AppClaims: Claims>(
        self,
        extractors: Extractors<AppClaims>,
        pool: Option<Pool>,
    ) -> App<
        impl ServiceFactory<
            dev::ServiceRequest,
            Error = actix_web::Error,
            Config = (),
            InitError = (),
            Response = dev::ServiceResponse,
        >,
    > {
        let mut builder =
            SessionMiddlewareFactory::build_ed_dsa().with_extractors(extractors.clone());
        if let Some(pool) = pool {
            builder = builder.with_redis_pool(pool);
        }
        // create new [SessionStorage] and [SessionMiddlewareFactory]
        let (storage, factory) = builder.finish();
        self.app_data(Data::new(extractors.clone()))
            .app_data(Data::new(storage))
            .wrap(factory)
    }
}
