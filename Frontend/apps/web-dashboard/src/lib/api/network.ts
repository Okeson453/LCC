/**
 * Network (CRM) API — contacts, companies, interactions.
 *
 * Canonical paths from `config.ts::API_PATHS.contacts`.
 */

import { apiFetch } from './client';
import { API_PATHS } from './config';
import type {
  Contact,
  ContactUpdate,
  Interaction,
  InteractionCreate,
  RelationshipStage,
} from '@lcc/api-types';

export async function listContacts(
  _memberId: string,
  filter?: { stage?: RelationshipStage; expand?: 'company' },
): Promise<Contact[]> {
  return apiFetch<Contact[]>(API_PATHS.contacts.list, { query: filter });
}

export async function getContact(_memberId: string, contactId: string): Promise<Contact> {
  return apiFetch<Contact>(API_PATHS.contacts.item(contactId));
}

export async function updateContact(
  _memberId: string,
  contactId: string,
  update: ContactUpdate,
): Promise<Contact> {
  return apiFetch<Contact>(API_PATHS.contacts.item(contactId), {
    method: 'PATCH',
    body: update,
  });
}

export async function getStaleContacts(_memberId: string): Promise<Contact[]> {
  return apiFetch<Contact[]>(API_PATHS.contacts.staleness);
}

export async function listContactInteractions(
  _memberId: string,
  contactId: string,
): Promise<Interaction[]> {
  return apiFetch<Interaction[]>(API_PATHS.contacts.interactions(contactId));
}

export async function logInteraction(
  _memberId: string,
  contactId: string,
  input: InteractionCreate,
): Promise<Interaction> {
  return apiFetch<Interaction>(API_PATHS.contacts.interactions(contactId), {
    method: 'POST',
    body: input,
  });
}
