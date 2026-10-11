use std::borrow::Cow;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct User<'a> {
    pub id: Ulid,
    pub created_at: time::OffsetDateTime,
    pub name: Cow<'a, str>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Submission {
    pub id: Ulid,
    pub status: SubmissionStatus,
    pub created_at: time::OffsetDateTime,
    /// Points at a user
    pub created_by: Ulid,
}

#[derive(Clone, Copy, Debug, sqlx::Type, Serialize)]
#[sqlx(type_name = "submission_status", rename_all = "lowercase")]
pub enum SubmissionStatus {
    Draft = 0,
    Queued = 1,
    Running = 2,
    Complete = 3,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[repr(transparent)]
pub struct Ulid(pub ulid::Ulid);

#[derive(Clone, Debug, Serialize)]
pub struct GithubAssociation<'a> {
    pub github_user_id: i64,
    pub github_email: Cow<'a, str>,
    pub github_name: Cow<'a, str>,
    pub created_at: time::OffsetDateTime,
    /// Points at a user
    pub user_id: Ulid,
}

impl sqlx::Type<sqlx::Postgres> for Ulid {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

// impl sqlx::postgres::PgHasArrayType for Ulid {
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         <&Vec<uuid::Uuid> as sqlx::Type<sqlx::Postgres>>::type_info()
//     }
//     fn array_compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
//         <&uuid::Uuid as sqlx::Type<sqlx::Postgres>>::compatible(ty)
//     }
// }

impl sqlx::Encode<'_, sqlx::Postgres> for Ulid {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let mut str_buff = [0; ulid::ULID_LEN];
        let ulid_str = self.0.array_to_str(&mut str_buff);
        buf.extend_from_slice(ulid_str.as_bytes());

        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for Ulid {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let ulid = ulid::Ulid::from_string(value.as_str()?)?;

        Ok(Ulid(ulid))
    }
}
