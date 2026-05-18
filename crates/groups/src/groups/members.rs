use db::AppError;
use db::GroupMemberDetail;
use db::entities;
use db::entities::enums::{GroupRole, P2pRequestStatus};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, Set,
};

pub async fn list_group_members(
    db: &DatabaseConnection,
    user_id: &str,
    group_id: &str,
) -> Result<Vec<GroupMemberDetail>, AppError> {
    // Verify membership to prevent IDOR
    let is_member =
        entities::user_groups::Entity::find_by_id((user_id.to_string(), group_id.to_string()))
            .one(db)
            .await?;

    if is_member.is_none() {
        return Err(AppError::unauthorized("User is not a member of this group"));
    }

    // Join user_groups with users to get member details
    let results = entities::user_groups::Entity::find()
        .filter(entities::user_groups::Column::GroupId.eq(group_id.to_string()))
        .join(
            JoinType::InnerJoin,
            entities::user_groups::Relation::Users.def(),
        )
        .column_as(entities::users::Column::Name, "name")
        .column_as(entities::users::Column::Email, "email")
        .column_as(entities::user_groups::Column::UserId, "user_id")
        .column_as(entities::user_groups::Column::Role, "role")
        .into_model::<GroupMemberDetail>()
        .all(db)
        .await?;

    Ok(results)
}

pub async fn invite_to_group(
    db: &DatabaseConnection,
    sender_id: &str,
    receiver_email: &str,
    group_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let group = entities::groups::Entity::find_by_id(group_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Group not found"))?;

    let request = entities::p2p_requests::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        sender_user_id: Set(sender_id.to_string()),
        receiver_email: Set(receiver_email.to_string()),
        transaction_data: Set(serde_json::json!({
            "type": "GROUP_INVITE",
            "group_id": group.id,
            "group_name": group.name
        })),
        status: Set(P2pRequestStatus::GroupInvite),
        linked_txn_id: Set(None),
    };

    request.insert(db).await.map_err(AppError::from)
}

pub async fn remove_group_member(
    db: &DatabaseConnection,
    admin_id: &str,
    group_id: &str,
    target_user_id: &str,
) -> Result<(), AppError> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::unauthorized("Admin not in group"))?;

    if admin_membership.role != GroupRole::Admin {
        return Err(AppError::unauthorized("Insufficient permissions"));
    }

    entities::user_groups::Entity::delete_by_id((target_user_id.to_string(), group_id.to_string()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn update_member_role(
    db: &DatabaseConnection,
    admin_id: &str,
    group_id: &str,
    target_user_id: &str,
    new_role: GroupRole,
) -> Result<(), AppError> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::unauthorized("Admin not in group"))?;

    if admin_membership.role != GroupRole::Admin {
        return Err(AppError::unauthorized("Insufficient permissions"));
    }

    let mut membership: entities::user_groups::ActiveModel =
        entities::user_groups::Entity::find_by_id((
            target_user_id.to_string(),
            group_id.to_string(),
        ))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Member not found"))?
        .into();

    membership.role = Set(new_role);
    membership.update(db).await?;
    Ok(())
}
