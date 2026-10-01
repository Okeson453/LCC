import { Metadata } from 'next';
import { ProfileShellClient } from '../_components/profile-shell-client';

export const metadata: Metadata = { title: 'Profile audit' };

export default function AuditPage(): React.ReactElement {
  return <ProfileShellClient />;
}
