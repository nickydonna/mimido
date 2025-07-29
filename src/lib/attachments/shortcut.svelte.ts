import type { Attachment } from "svelte/attachments";

type Options = {
  optionOrAlt?: boolean,
  control?: boolean,
  cmd?: boolean,
  shift?: boolean,
  key: string,
  handler: () => void,
};

export function shortcut({ optionOrAlt, control, cmd, shift, key, handler }: Options): Attachment {
  const callback = (e: KeyboardEvent) => {
    const optionCheck = optionOrAlt === true ? e.altKey : true;
    const controlCheck = control === true ? e.ctrlKey : true;
    const shiftCheck = shift === true ? e.shiftKey : true;
    const metaCheck = cmd === true ? e.metaKey : true;
    const keyCheck = e.key === key;

    if (optionCheck && controlCheck && shiftCheck && metaCheck && keyCheck) {
      handler()
    }

  }
  return () => {
    window.addEventListener('keydown', callback)
    return () => {

      window.removeEventListener('keydown', callback)
    }
  }
}
