import { Metadata } from 'next';
import { NetworkClient } from './_components/network-client';

export const metadata: Metadata = { title: 'Network' };

export default function NetworkPage(): React.ReactElement {
  return <NetworkClient />;
}
