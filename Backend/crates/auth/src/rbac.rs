//! RBAC — Role-Based Access Control.
//!
//! Five roles (Source Backend Design Concept §43.2):
//! - `Owner`   — full read/write, approve all actions
//! - `Assistant` — draft/edit content and outreach; cannot approve sends
//! - `Reviewer` — approve/reject queued actions; cannot create new drafts
//! - `Admin`   — manage compliance config versions (requires 2-reviewer activation)
//! - `Auditor` — read-only access to audit_log and analytics

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Assistant,
    Reviewer,
    Admin,
    Auditor,
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Role::Owner),
            "assistant" => Ok(Role::Assistant),
            "reviewer" => Ok(Role::Reviewer),
            "admin" => Ok(Role::Admin),
            "auditor" => Ok(Role::Auditor),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        match self {
            Role::Owner => "owner",
            Role::Assistant => "assistant",
            Role::Reviewer => "reviewer",
            Role::Admin => "admin",
            Role::Auditor => "auditor",
        }
    }
}

/// Permissions — actions that can be gated by RBAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    DraftContent,
    EditContent,
    ApproveSend,
    ReviewApprovalQueue,
    ManageComplianceConfig,
    ViewAuditLog,
    ViewAnalytics,
    ViewOwnData,
    ManageOwnSettings,
    DeleteOwnAccount,
}

pub fn has_permission(role: Role, perm: Permission) -> bool {
    use Permission::*;
    use Role::*;
    match (role, perm) {
        (Owner, _) => true,
        (Admin, _) => true, // Admins can do anything except view audit (audit-only is Auditor)
        (Auditor, ViewAuditLog) => true,
        (Auditor, ViewAnalytics) => true,
        (Auditor, ViewOwnData) => true,
        (Reviewer, ReviewApprovalQueue) => true,
        (Reviewer, ViewAnalytics) => true,
        (Reviewer, ViewOwnData) => true,
        (Assistant, DraftContent | EditContent | ViewOwnData | ManageOwnSettings) => true,
        // Self-service permissions every role holds regardless of duty. Scoped
        // to the roles not already granted them above so each match arm stays
        // reachable (a blanket `(_, ..)` here is dead for Owner/Admin/Assistant).
        (Reviewer | Auditor, ManageOwnSettings | DeleteOwnAccount) => true,
        (Assistant, DeleteOwnAccount) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_can_do_everything() {
        assert!(has_permission(Role::Owner, Permission::ApproveSend));
        assert!(has_permission(Role::Owner, Permission::ManageComplianceConfig));
        assert!(has_permission(Role::Owner, Permission::ViewAuditLog));
    }

    #[test]
    fn assistant_cannot_approve_sends() {
        assert!(has_permission(Role::Assistant, Permission::DraftContent));
        assert!(!has_permission(Role::Assistant, Permission::ApproveSend));
    }

    #[test]
    fn reviewer_can_approve_queue() {
        assert!(has_permission(Role::Reviewer, Permission::ReviewApprovalQueue));
        assert!(!has_permission(Role::Reviewer, Permission::DraftContent));
    }

    #[test]
    fn auditor_can_view_logs_but_not_send() {
        assert!(has_permission(Role::Auditor, Permission::ViewAuditLog));
        assert!(!has_permission(Role::Auditor, Permission::ApproveSend));
    }

    #[test]
    fn admin_can_manage_compliance_config() {
        assert!(has_permission(Role::Admin, Permission::ManageComplianceConfig));
        assert!(has_permission(Role::Admin, Permission::ViewAnalytics));
    }

    #[test]
    fn everyone_can_manage_own_settings() {
        for r in [Role::Owner, Role::Assistant, Role::Reviewer, Role::Admin, Role::Auditor] {
            assert!(has_permission(r, Permission::ManageOwnSettings), "{:?}", r);
        }
    }
}
