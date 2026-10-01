/**
 * @lcc/ui — public exports.
 *
 * Apps import from `@lcc/ui` and never reach into `./primitives/*` directly.
 */

// Primitives
export * from './primitives/button';
export * from './primitives/input';
export * from './primitives/textarea';
export * from './primitives/label';
export * from './primitives/badge';
export * from './primitives/card';
export * from './primitives/separator';
export * from './primitives/skeleton';
export * from './primitives/dialog';
export * from './primitives/sheet';
export * from './primitives/tooltip';
export * from './primitives/tabs';
export * from './primitives/select';
export * from './primitives/checkbox';
export * from './primitives/radio-group';
export * from './primitives/switch';
export * from './primitives/slider';
export * from './primitives/dropdown-menu';
export * from './primitives/context-menu';
export * from './primitives/menubar';
export * from './primitives/navigation-menu';
export * from './primitives/popover';
export * from './primitives/avatar';
export * from './primitives/progress';
export * from './primitives/alert';
export * from './primitives/toast';
export * from './primitives/toaster';
export * from './primitives/sonner';
export * from './primitives/accordion';
export * from './primitives/collapsible';
export * from './primitives/command';
export * from './primitives/combobox';
export * from './primitives/calendar';
export * from './primitives/date-picker';
export * from './primitives/form';
export * from './primitives/table';
export * from './primitives/pagination';
export * from './primitives/breadcrumb';
export * from './primitives/scroll-area';
export * from './primitives/resizable';
export * from './primitives/aspect-ratio';

// Patterns
export * from './patterns/empty-state';
export * from './patterns/error-state';
export * from './patterns/loading-skeleton';
export * from './patterns/confirm-dialog';
export * from './patterns/data-table';
export * from './patterns/keyboard-shortcut';
export * from './patterns/copy-to-clipboard';
export * from './patterns/VirtualList';

// Theme
export * from './theme/ThemeProvider';
export * from './theme/use-theme';
export * from './theme/mode-toggle';

// Icons
export * from './icons';

// A11y
export * from './a11y/skip-nav';
export * from './a11y/live-region';
export * from './a11y/focus-trap';
export * from './a11y/visually-hidden';

// Utils
export * from './utils';
