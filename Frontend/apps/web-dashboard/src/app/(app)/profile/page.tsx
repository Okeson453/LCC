import { Metadata } from 'next';
import { ProfileShellClient } from './_components/profile-shell-client';

export const metadata: Metadata = { title: 'Profile' };

export default function ProfilePage(): React.ReactElement {
  return <ProfileShellClient />;
}
