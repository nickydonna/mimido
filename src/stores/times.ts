import { readable } from "svelte/store";

/**
 * Svelte Store that ticks every minute
 */
export const timeStore = readable(new Date(), (set) => {
  set(new Date());

  const interval = setInterval(() => {
    set(new Date());
  }, 10000);

  return () => clearInterval(interval);
});
