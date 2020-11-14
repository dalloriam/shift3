use std::convert::TryFrom;
use std::fmt;
use std::mem;

use google_cloud::{datastore, error::Error as GCloudError};

use snafu::{ResultExt, Snafu};

use crate::{
    datastore::{DSEntity, DatastoreEntity, EntityConversionError},
    https, AuthProvider,
};

#[derive(Debug, Snafu)]
pub enum DatastoreError {
    #[snafu(display("Failed to create client: {}", source))]
    FailedToCreateClient {
        source: GCloudError,
    },

    InsertFailed {
        source: GCloudError,
    },
    QueryFailed {
        message: String,
    },
    BadEntity {
        source: EntityConversionError,
    },
    IncompleteData,
}

type Result<T> = std::result::Result<T, DatastoreError>;

/// High-level client for Google Datastore.
pub struct DatastoreClient {
    client: datastore::Client,
}

impl DatastoreClient {
    /// Returns a new client from a project ID and an authenticator.
    pub async fn new(project_id: String, authenticator: AuthProvider) -> Result<DatastoreClient> {
        let client = datastore::Client::from_credentials(project_id, authenticator.into()).await?;
        Ok(DatastoreClient { client })
    }

    /// Sets an optional batch size for the client.
    ///
    /// The batch size dictates the maximum number of pending mutations
    /// between commits.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self
    }

    /// Returns whether the client has unapplied mutations.
    #[inline]
    pub fn has_pending_operations(&self) -> bool {
        unimplemented!();
    }

    /// Commits the pending mutation buffer.
    pub fn commit(&mut self) -> Result<()> {
        unimplemented!()
        /*
        if !self.has_pending_operations() {
            // Short-circuit immediately
            // if no pending operations.
            return Ok(());
        }

        // Take the mutation buffer from the current instance
        // and replace it with an empty buffer.
        let mut new_mutation_buffer = Vec::new();
        mem::swap(&mut self.mutation_buffer, &mut new_mutation_buffer);

        let request = CommitRequest {
            mode: Some(CommitMode::NonTransactional.to_string()),
            transaction: None,
            mutations: Some(new_mutation_buffer),
        };

        // For now, mutation results are discarded.
        let (_client_response, _commit_response) = self
            .hub
            .projects()
            .commit(request, &self.project_id)
            .doit()
            .map_err(|e| DatastoreError::InsertFailed {
                message: e.to_string(),
            })?;

        Ok(())
        */
    }

    fn insert_ds(&mut self, ds_entity: DSEntity) -> Result<()> {
        unimplemented!()
        /*
        self.mutation_buffer.push(Mutation {
            insert: Some(ds_entity.into()),
            ..Default::default()
        });

        // If the last mutation overflowed the mutation buffer,
        // commit the pending mutations.
        if let Some(batch_size) = self.batch_size {
            if self.mutation_buffer.len() > batch_size {
                self.commit()?;
            }
        }
        Ok(())
        */
    }

    /// Inserts a new entity to datastore.
    ///
    /// This method actually inserts a mutation to the mutation buffer, which must be committed
    /// before dropping the client for fear of skipping operations.
    pub async fn insert<T: datastore::IntoEntity>(&mut self, item: T) -> Result<()> {
        unimplemented!()
    }

    /// Inserts a new entity to datastore, specifying an explicit ID.
    pub fn insert_with_name<T: DatastoreEntity>(&mut self, name: String, item: T) -> Result<()> {
        let mut ds_entity = item.into_entity();
        ds_entity.entity_name = Some(name);
        self.insert_ds(ds_entity)
    }

    /// Gets all items of a given type.
    pub fn get_all<T>(&self) -> Result<Vec<T>>
    where
        T: DatastoreEntity,
    {
        unimplemented!();
        /*
        let query = Query {
            kind: Some(vec![KindExpression {
                name: Some(String::from(T::get_kind())),
            }]),
            offset: Some(0),
            limit: Some(100),
            ..Default::default()
        };

        let (_resp, r) = self
            .hub
            .projects()
            .run_query(
                RunQueryRequest {
                    query: Some(query),
                    ..Default::default()
                },
                &self.project_id,
            )
            .doit()
            .map_err(|e| DatastoreError::QueryFailed {
                message: e.to_string(),
            })?;

        let batch = r.batch.ok_or(DatastoreError::IncompleteData)?;
        let entities = batch.entity_results.ok_or(DatastoreError::IncompleteData)?;

        let mut results = Vec::new();
        for entity_result in entities.into_iter() {
            let entity = entity_result.entity.ok_or(DatastoreError::IncompleteData)?;
            let user_type = T::from_entity(DSEntity::try_from(entity).context(BadEntity)?);
            results.push(user_type);
        }

        Ok(results)
        */
    }

    /// Get an entity by its id.
    pub fn get<T>(&self, id: u64) -> Result<Option<T>>
    where
        T: DatastoreEntity,
    {
        unimplemented!();
        /*
            let query = Query {
                kind: Some(vec![KindExpression {
                    name: Some(String::from(T::get_kind())),
                }]),
                filter: Some(Filter {
                    property_filter: Some({
                        PropertyFilter {
                            property: Some(PropertyReference {
                                name: Some(String::from("id")),
                            }),
                            value: Some(Value {
                                integer_value: Some(id.to_string()),
                                ..Default::default()
                            }),
                            op: Some(String::from("EQUAL")),
                        }
                    }),
                    ..Default::default()
                }),
                offset: Some(0),
                limit: Some(1),
                ..Default::default()
            };

            let (_resp, r) = self
                .hub
                .projects()
                .run_query(
                    RunQueryRequest {
                        query: Some(query),
                        ..Default::default()
                    },
                    &self.project_id,
                )
                .doit()
                .map_err(|e| DatastoreError::QueryFailed {
                    message: e.to_string(),
                })?;

            let batch = r.batch.ok_or(DatastoreError::IncompleteData)?;
            let entities = batch.entity_results.ok_or(DatastoreError::IncompleteData)?;

            let mut results = Vec::new();
            for entity_result in entities.into_iter() {
                let entity = entity_result.entity.ok_or(DatastoreError::IncompleteData)?;
                let user_type = T::from_entity(DSEntity::try_from(entity).context(BadEntity)?);
                results.push(user_type);
            }

            if results.is_empty() {
                Ok(None)
            } else {
                // Only keep the first element
                Ok(Some(results.remove(0)))
            }
        */
    }
}

impl Drop for DatastoreClient {
    fn drop(&mut self) {
        unimplemented!();
        /*
        // Validate that nothing is in the tx buffer.
        if !self.mutation_buffer.is_empty() {
            log::error!("transaction buffer not empty")
        }
        */
    }
}
