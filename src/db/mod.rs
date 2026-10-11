use crate::model::*;

#[derive(Clone)]
pub struct DbClient {
    db: sqlx::PgPool,
}

impl DbClient {
    pub async fn new(conn: sqlx::PgPool) -> Self {
        DbClient { db: conn }
    }

    pub async fn create_user(&self, id: ulid::Ulid, name: &str) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, name)
            VALUES ($1, $2)
            "#,
            Ulid(id) as Ulid,
            name,
        )
        .execute(&self.db)
        .await
        .map(|_| ())
    }

    pub async fn get_user<'a>(&'a self, id: ulid::Ulid) -> sqlx::Result<Option<User<'a>>> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: _", name, created_at FROM users
            WHERE id = $1
            "#,
            Ulid(id) as Ulid
        )
        .fetch_optional(&self.db)
        .await
    }

    // TODO: rework to streams
    pub async fn get_user_submissions(&self, user_id: ulid::Ulid) -> sqlx::Result<Vec<Submission>> {
        // NOTE: id is an ulid. Ulids are sortable chronologically
        sqlx::query_as!(
            Submission,
            r#"
            SELECT id as "id: _", status as "status: _", created_at, created_by as "created_by: _" FROM submissions
            WHERE created_by = $1
            ORDER BY id DESC
            "#,
            Ulid(user_id) as Ulid
        ).fetch_all(&self.db).await
    }

    pub async fn create_submission(&self, id: ulid::Ulid, author: ulid::Ulid) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO submissions ( id, created_by )
            VALUES ($1, $2 )
            "#,
            Ulid(id) as Ulid,
            Ulid(author) as Ulid,
        )
        .execute(&self.db)
        .await
        .map(|_| ())
    }

    pub async fn set_submission_status(
        &self,
        id: ulid::Ulid,
        status: SubmissionStatus,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            UPDATE submissions
            SET status = $2
            WHERE id = $1
            "#,
            Ulid(id) as Ulid,
            status as SubmissionStatus,
        )
        .execute(&self.db)
        .await
        .map(|_| ())
    }

    pub async fn get_submission(&self, id: ulid::Ulid) -> sqlx::Result<Option<Submission>> {
        sqlx::query_as!(
            Submission,
            r#"
            SELECT id as "id: _", status as "status: _", created_at, created_by as "created_by: _" FROM submissions
            WHERE id = $1
            "#,
            Ulid(id) as Ulid
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn create_github_association(
        &self,
        github_user_id: i64,
        github_email: &str,
        github_name: &str,
        user_id: ulid::Ulid,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO github_associations (github_user_id, github_email, github_name, user_id)
            VALUES ($1, $2, $3, $4)
            "#,
            github_user_id,
            github_email,
            github_name,
            Ulid(user_id) as Ulid,
        )
        .execute(&self.db)
        .await
        .map(|_| ())
    }

    pub async fn get_github_association<'a>(
        &'a self,
        github_user_id: i64,
    ) -> sqlx::Result<Option<GithubAssociation<'a>>> {
        sqlx::query_as!(
            GithubAssociation,
            r#"
            SELECT github_user_id, github_email, github_name, created_at, user_id as "user_id: _" FROM github_associations 
            WHERE github_user_id = $1
            "#,
            github_user_id
        )
        .fetch_optional(&self.db)
        .await
    }

    // TODO: a transaction for GitHub-based registration
}
