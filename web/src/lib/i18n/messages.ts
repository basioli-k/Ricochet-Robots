/** English is the reference locale: every other locale must have the same keys. */
export const en = {
  appTitle: 'Ricochet Robots',
  comingSoon: 'The new version is under construction.',
} as const;

export type MessageKey = keyof typeof en;
export type Messages = Record<MessageKey, string>;

export const hr: Messages = {
  appTitle: 'Ricochet Robots',
  comingSoon: 'Nova verzija je u izradi.',
};
