import { Metadata } from 'next';
import { NetworkClient } from '../_components/network-client';

export const metadata: Metadata = { title: 'Companies' };

export default function CompaniesPage(): React.ReactElement {
  return <NetworkClient />;
}
