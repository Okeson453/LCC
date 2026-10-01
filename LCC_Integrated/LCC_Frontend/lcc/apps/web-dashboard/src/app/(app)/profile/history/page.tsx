import { Metadata } from 'next';
import { ProfileShellClient } from '../_components/profile-shell-client';

export const metadata: Metadata = { title: 'Profile history' };

export default function HistoryPage(): React.ReactElement {
  return <ProfileShellClient />;
}
