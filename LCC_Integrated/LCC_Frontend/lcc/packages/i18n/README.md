# @lcc/i18n

i18n setup using `next-intl`. English is the launch locale; structure is in place for additional locales.

## Architecture

```
@lcc/i18n/
├── src/
│   ├── index.ts           # public exports
│   ├── config.ts          # locale list, default, fallback
│   ├── provider.tsx       # <I18nProvider>
│   ├── use-translation.ts # typed useTranslations wrapper
│   └── locales/
│       ├── en/            # English (launch)
│       │   ├── common.json
│       │   ├── approval.json
│       │   ├── content.json
│       │   ├── engagement.json
│       │   ├── outreach.json
│       │   ├── opportunity.json
│       │   ├── analytics.json
│       │   ├── copilot.json
│       │   └── errors.json
│       └── (es|de|fr)/    # future locales
```

## Usage

```tsx
import { useTranslation } from '@lcc/i18n';

function ApprovalTitle() {
  const t = useTranslation('approval');
  return <h1>{t('queue.title')}</h1>;
}
```
