import { type Event, type EventType } from '../bindings';

export type ParsedEvent = Omit<Event, "starts_at" | "ends_at"> & {
  starts_at: Date;
  ends_at: Date;
};



export const getEventCardClass = (function (event: ParsedEvent) {
  const { tag, event_type } = event;
  const tags = tag?.split(',') ?? [];
  const isBlock = event_type === 'Block';
  const opacity = !isBlock ? 'bg-opacity-45' : '';
  const lcTags = tags.map((t) => t.toLowerCase());
  const bgTag = lcTags.find((t) => t.startsWith('bg:'));
  if (bgTag) {
    return `card__bg-${bgTag.replace('bg:', '')}`;
  }

  const color = getEventColor(event);
  return isBlock
    ? `${opacity} bg-polka-${color}-800 border-${color}-900`
    : `${opacity} bg-${color}-600 border-${color}-900`;
});

export const getEventColor = (function (event: ParsedEvent) {
  const EDefaultEventColor: Record<EventType, string> = {
    "Block": 'indigo',
    "Event": 'green',
    "Task": 'pink',
    "Reminder": 'blue'
  };
  const { tag, event_type } = event;

  const lcTags = tag?.split(',').map((t) => t.toLowerCase()) ?? [];
  const colorTag = lcTags.find((t) => t.startsWith('c:'));
  return colorTag?.replace('c:', '') ?? EDefaultEventColor[event_type];
});
