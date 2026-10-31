// NOEMA Brain full-viewport hash route controller.
// Manages #brain route, Escape dismissal, reduced motion, and clean coexistence with #studio.

export interface BrainRouteHooks {
  setBrainMode: (on: boolean) => void;
  isBrainMode: () => boolean;
}

export class BrainRoute {
  private hooks: BrainRouteHooks;
  private keydownHandler: ((e: KeyboardEvent) => void) | null = null;
  private hashHandler: (() => void) | null = null;
  private mediaQuery: MediaQueryList | null = null;
  private mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;
  reducedMotion = false;

  constructor(hooks: BrainRouteHooks) {
    this.hooks = hooks;
  }

  init(): void {
    if (typeof window === 'undefined') return;

    // Reduced motion preference
    this.mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    this.reducedMotion = this.mediaQuery.matches;
    this.mediaHandler = (e: MediaQueryListEvent) => {
      this.reducedMotion = e.matches;
    };
    this.mediaQuery.addEventListener('change', this.mediaHandler);

    // Escape key dismisses #brain back to default
    this.keydownHandler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && this.isOpen()) {
        e.preventDefault();
        this.close();
      }
    };
    window.addEventListener('keydown', this.keydownHandler);

    // Hash route listener
    this.hashHandler = () => {
      this.applyHash();
    };
    window.addEventListener('hashchange', this.hashHandler);

    this.applyHash();
  }

  isOpen(): boolean {
    return this.hooks.isBrainMode();
  }

  open(): void {
    if (location.hash.replace(/^#\/?/, '') !== 'brain') {
      location.hash = 'brain';
    } else {
      this.hooks.setBrainMode(true);
    }
  }

  close(): void {
    if (location.hash.replace(/^#\/?/, '') === 'brain') {
      history.replaceState(null, '', location.pathname + location.search);
    }
    this.hooks.setBrainMode(false);
  }

  applyHash(): void {
    const route = location.hash.replace(/^#\/?/, '');
    if (route === 'brain') {
      if (!this.isOpen()) {
        this.hooks.setBrainMode(true);
      }
    } else if (this.isOpen()) {
      // Navigated to #studio, empty or another route
      this.hooks.setBrainMode(false);
    }
  }

  destroy(): void {
    if (this.keydownHandler) window.removeEventListener('keydown', this.keydownHandler);
    if (this.hashHandler) window.removeEventListener('hashchange', this.hashHandler);
    if (this.mediaQuery && this.mediaHandler) {
      this.mediaQuery.removeEventListener('change', this.mediaHandler);
    }
  }
}

/**
 * 5-line integration snippet for main.ts (Aesthesis):
 *
 * import { mountBrainRoute } from './brain-route';
 * const brainRoute = mountBrainRoute(setBrainMode, () => brainMode);
 * (window as any).__noemaBrainRoute = brainRoute;
 */
export function mountBrainRoute(setBrainMode: (on: boolean) => void, isBrainMode: () => boolean): BrainRoute {
  const route = new BrainRoute({ setBrainMode, isBrainMode });
  route.init();
  return route;
}
