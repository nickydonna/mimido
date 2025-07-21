import type { Attachment } from "svelte/attachments";

type Options = {
  rootMargin?: string;
  threshold?: number | number[];
  onEnter?: () => void,
  onExit?: () => void,
};

export function inview({ onEnter, onExit, rootMargin = '0px', threshold = 0 }: Options): Attachment {
  return (element) => {
    console.log('start');

    const observer = new IntersectionObserver((entries) => {
      entries.forEach(singleEntry => {
        if (singleEntry.isIntersecting) {
          onEnter?.();
        } else {
          onExit?.();
        }
      })
    }, {
      threshold, rootMargin
    })
    observer.observe(element)

    return () => {
      observer.unobserve(element)
    }

  }
}
