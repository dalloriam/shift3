use std::convert::TryFrom;
use std::fmt;
use std::mem;

use google_datastore1::{
    CommitRequest, Datastore, Error as GoogleDsError, KindExpression, Mutation, Query,
    RunQueryRequest,
};

use hyper::Client;

use snafu::{ResultExt, Snafu};

use protocol::{Rule, RuleID};

use crate::{
    datastore::{DSEntity, DatastoreEntity, EntityConversionError},
    https, AuthProvider,
};

enum CommitMode {
    NonTransactional,
}

impl fmt::Display for CommitMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &'static str = match self {
            CommitMode::NonTransactional => "NON_TRANSACTIONAL",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Snafu)]
pub enum DatastoreError {
    InsertFailed { source: GoogleDsError },
    QueryFailed { source: GoogleDsError },
    BadEntity { source: EntityConversionError },
    IncompleteData,
}

type Result<T> = std::result::Result<T, DatastoreError>;

/// High-level client for Google Datastore.
pub struct DatastoreClient {
    batch_size: Option<usize>,

    hub: Datastore<Client, AuthProvider>,

    mutation_buffer: Vec<Mutation>,

    project_id: String,
}

impl DatastoreClient {
    /// Returns a new client from a project ID and an authenticator.
    pub fn new(project_id: String, authenticator: AuthProvider) -> DatastoreClient {
        let hub = Datastore::new(https::new_tls_client(), authenticator);
        DatastoreClient {
            batch_size: None,
            hub,
            mutation_buffer: Vec::new(),
            project_id,
        }
    }

    /// Sets an optional batch size for the client.
    ///
    /// The batch size dictates the maximum number of pending mutations
    /// between commits.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    /// Returns whether the client has unapplied mutations.
    #[inline]
    pub fn has_pending_operations(&self) -> bool {
        !self.mutation_buffer.is_empty()
    }

    /// Commits the pending mutation buffer.
    pub fn commit(&mut self) -> Result<()> {
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
            .context(InsertFailed)?;

        Ok(())
    }

    fn insert_ds(&mut self, ds_entity: DSEntity) -> Result<()> {
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
    }

    /// Inserts a new entity to datastore.
    ///
    /// This method actually inserts a mutation to the mutation buffer, which must be committed
    /// before dropping the client for fear of skipping operations.
    pub fn insert<T: DatastoreEntity>(&mut self, item: T) -> Result<()> {
        let ds_entity = item.into_entity();
        self.insert_ds(ds_entity)
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
            .context(QueryFailed)?;

        let batch = r.batch.ok_or(DatastoreError::IncompleteData)?;
        let entities = batch.entity_results.ok_or(DatastoreError::IncompleteData)?;

        let mut results = Vec::new();
        for entity_result in entities.into_iter() {
            let entity = entity_result.entity.ok_or(DatastoreError::IncompleteData)?;
            let user_type = T::from_entity(DSEntity::try_from(entity).context(BadEntity)?);
            results.push(user_type);
        }

        Ok(results)
    }

    /// Get a rule by its id.
    pub fn get(&self, id: RuleID) -> Result<Rule> {
        // TODO: Implement this!
        unimplemented!()
    }
}

impl Drop for DatastoreClient {
    fn drop(&mut self) {
        // Validate that nothing is in the tx buffer.
        if !self.mutation_buffer.is_empty() {
            eprintln!("Warning: Transaction buffer not empty"); // TODO: Replace with `log` library.
        }
    }
}
