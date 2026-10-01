import { Metadata } from 'next';
import { ProfileShellClient } from '../_components/profile-shell-client';

export const metadata: Metadata = { title: 'Profile edits' };

export default function EditsPage(): React.ReactElement {
  return <ProfileShellClient />;
}
