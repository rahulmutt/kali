use crate::*;

#[test]
fn permissions_query_reports_granted_and_denied() {
    let permissions = DenoPermissions::new(true, false, true, false);
    assert_eq!(
        permissions.query(DenoPermissionKind::Read),
        Ok(DenoPermissionStatus::Granted)
    );
    assert_eq!(
        permissions.query(DenoPermissionKind::Write),
        Ok(DenoPermissionStatus::Denied)
    );
    assert_eq!(
        permissions.query(DenoPermissionKind::Net),
        Ok(DenoPermissionStatus::Granted)
    );
    assert_eq!(
        permissions.query(DenoPermissionKind::Env),
        Ok(DenoPermissionStatus::Denied)
    );
    assert!(
        matches!(permissions.request(DenoPermissionKind::Read), Err(err) if err.to_string().contains("request"))
    );
    assert!(
        matches!(permissions.revoke(DenoPermissionKind::Read), Err(err) if err.to_string().contains("revoke"))
    );
}
