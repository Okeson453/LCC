/**
 * Onboarding store — wizard step, KB records pending.
 */

import { create } from 'zustand';

interface OnboardingState {
  currentStep: 'kb' | 'voice' | 'review' | 'goals' | 'audit' | 'done';
  setStep: (step: OnboardingState['currentStep']) => void;
  goalMode: 'job_hunting' | 'client_acquisition' | 'hybrid' | null;
  setGoalMode: (mode: OnboardingState['goalMode']) => void;
  resumeFileName: string | null;
  setResumeFileName: (name: string | null) => void;
  voiceSampleIds: string[];
  addVoiceSampleId: (id: string) => void;
  portfolioLinks: string[];
  addPortfolioLink: (link: string) => void;
}

export const useOnboardingStore = create<OnboardingState>((set) => ({
  currentStep: 'kb',
  setStep: (currentStep) => set({ currentStep }),
  goalMode: null,
  setGoalMode: (goalMode) => set({ goalMode }),
  resumeFileName: null,
  setResumeFileName: (resumeFileName) => set({ resumeFileName }),
  voiceSampleIds: [],
  addVoiceSampleId: (id) =>
    set((s) => ({ voiceSampleIds: [...s.voiceSampleIds, id] })),
  portfolioLinks: [],
  addPortfolioLink: (link) =>
    set((s) => ({ portfolioLinks: [...s.portfolioLinks, link] })),
}));
