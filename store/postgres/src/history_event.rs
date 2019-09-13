//! A `HistoryEvent` is used to track entity operations that belong
//! together because they came from the same block in the JSONB storage
//! scheme
use diesel::deserialize::QueryableByName;
use diesel::pg::{Pg, PgConnection};
use diesel::sql_types::Text;
use diesel::RunQueryDsl;

use graph::prelude::{EthereumBlockPointer, SubgraphDeploymentId};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HistoryEvent {
    pub id: i32,
    pub subgraph: SubgraphDeploymentId,
    pub block_ptr: EthereumBlockPointer,
}

impl HistoryEvent {
    pub fn to_event_source_string(event: &Option<&HistoryEvent>) -> String {
        event.map_or(String::from("none"), |event| event.block_ptr.hash_hex())
    }

    /// Add an entry to the `event_meta_data` table for this `block_ptr` and
    /// return a `HistoryEvent` containing the id of that entry. This event
    /// should be used for all operations in the current transaction open on `conn`
    pub fn allocate(
        conn: &PgConnection,
        subgraph: SubgraphDeploymentId,
        block_ptr: EthereumBlockPointer,
    ) -> Result<HistoryEvent, failure::Error> {
        #[derive(Queryable, Debug)]
        struct Event {
            id: i32,
        };

        impl QueryableByName<Pg> for Event {
            fn build<R: diesel::row::NamedRow<Pg>>(row: &R) -> diesel::deserialize::Result<Self> {
                Ok(Event {
                    id: row.get("event_id")?,
                })
            }
        }

        let result: Event = diesel::sql_query(
            "insert into event_meta_data (db_transaction_id, db_transaction_time, source)
           values (txid_current(), statement_timestamp(), $1)
         returning event_meta_data.id as event_id",
        )
        .bind::<Text, _>(block_ptr.hash_hex())
        .get_result(conn)?;

        Ok(HistoryEvent {
            id: result.id,
            subgraph,
            block_ptr,
        })
    }
}