import adt, { match, def, type Variants } from "@korkje/adt";
import type { ScheduledTask, UnscheduledTask } from "$lib/util";

export const EventDragger = adt({
  None: null,
  Dragging: (event: ScheduledTask | UnscheduledTask) => ({ event }),
});

export const eventDragger = $state<{ state: Variants<typeof EventDragger> }>({
  state: EventDragger.None,
});

export type EventDraggerVariants = Variants<typeof EventDragger>;
export type DraggingVariant = ReturnType<typeof EventDragger.Dragging>;
export type NoneVariant = typeof EventDragger.None;

export function isDragging(
  dragger: EventDraggerVariants,
): dragger is DraggingVariant {
  return match(dragger, {
    Dragging: () => true,
    [def]: () => false,
  });
}
