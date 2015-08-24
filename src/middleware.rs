pub mod db {
    use std::any::Any;
    use std::error::Error;

    use iron::{IronResult, Request};
    use iron::middleware::BeforeMiddleware;
    use iron::typemap::Key;
    use plugin::Plugin;
    use r2d2;
    use r2d2::ManageConnection;

    #[derive(Clone)]
    pub struct ConnectionPool<M: Any + ManageConnection> {
        pool: r2d2::Pool<M>,
    }
    pub struct Connection<M: Any + ManageConnection> {
        _marker: ::std::marker::PhantomData<M>,
    }

    impl<M: Any + ManageConnection> ConnectionPool<M> {
        pub fn new(manager: M) -> Result<ConnectionPool<M>, Box<Error>> {
            let config = r2d2::Config::default();
            Ok(ConnectionPool { pool: try!(r2d2::Pool::new(config, manager)) })
        }
    }

    impl<M: Any + ManageConnection> Key for ConnectionPool<M> where
        M::Connection: Any, M::Error: Any
    {
        type Value = r2d2::Pool<M>;
    }

    impl<M: Any + ManageConnection> Key for Connection<M> where
        M::Connection: Any, M::Error: Any {
            type Value = r2d2::PooledConnection<M>;
        }

    impl<M: Any + ManageConnection> BeforeMiddleware for ConnectionPool<M> where
        M::Connection: Any, M::Error: Any
    {
        fn before(&self, req: &mut Request) -> IronResult<()> {
            req.extensions.insert::<ConnectionPool<M>>(self.pool.clone());
            Ok(())
        }
    }

    impl<'a, 'b, M: Any + ManageConnection> Plugin<Request<'a, 'b>> for Connection<M> where
        M::Connection: Any, M::Error: Any
    {
        type Error = r2d2::GetTimeout;

        fn eval(req: &mut Request) -> Result<Self::Value, Self::Error> {
            let pool = req.extensions.get::<ConnectionPool<M>>().unwrap();
            pool.get()
        }
    }
}
