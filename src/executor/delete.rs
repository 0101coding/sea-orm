use sea_query::{DeleteStatement, Expr, UpdateStatement};

use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, ModelTrait,
    Statement, StatementBuilder,
};
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Deleter<Q>
where
    Q: StatementBuilder,
{
    query: Q,
}

#[derive(Clone, Debug)]
pub struct DeleteResult {
    pub rows_affected: u64,
}

impl<'a, A: 'a> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only::<_, A::Entity>(self.query, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only::<_, E>(self.query, db)
    }
}

impl<Q> Deleter<Q>
where
    Q: StatementBuilder,
{
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only<'a, C, E>(
    delete_stmt: DeleteStatement,
    db: &'a C,
) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
    E: EntityTrait,
{
    let mut delete_stmt = delete_stmt;
    match <<E as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
        Some(soft_delete_column) => {
            let value = <E::Model as ModelTrait>::soft_delete_column_value();
            let update_stmt = UpdateStatement::new()
                .table(E::default())
                .col_expr(soft_delete_column, Expr::value(value))
                .set_conditions(delete_stmt.take_conditions())
                .to_owned();
            Deleter::new(update_stmt).exec(db).await
        }
        None => Deleter::new(delete_stmt).exec(db).await,
    }
}

async fn exec_delete<'a, C>(statement: Statement, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
