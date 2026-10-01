import { redirect } from 'next/navigation';

export default function OnboardingEntry(): never {
  redirect('/onboarding/kb');
}
