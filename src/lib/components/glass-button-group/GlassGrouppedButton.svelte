<script lang="ts">
  import type { SizeType } from "flowbite-svelte";
  import { getContext } from "svelte";
  import {
    type HTMLButtonAttributes,
    type HTMLAnchorAttributes,
  } from "svelte/elements";

  type BaseProps = {
    active?: boolean;
    disabled?: boolean;
  };

  type Props =
    | (HTMLAnchorAttributes & BaseProps)
    | (HTMLButtonAttributes & BaseProps);

  const sizes: Record<SizeType, string> = {
    xs: "p-[1px] text-xs",
    sm: "p-0.5 text-xs",
    md: "p-1.5 text-sm",
    lg: "p-2.5 text-sm",
    xl: "p-3 text-base",
  };

  let props: Props = $props();
  const { disabled: ctxDisabled, size } =
    getContext<() => { size: SizeType; disabled: boolean }>("group")();

  let { class: className, disabled: propDisabled, active } = $derived(props);
  let disabled = $derived(ctxDisabled || propDisabled);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  $effect(() => {
    console.log(size);
  });
  let classes = $derived([
    "glass-groupped-button",
    "rounded-3xl cursor-pointer",
    sizes[size],
    className,
    { active },
  ]);
</script>

{#if isButtonProps(props)}
  <button {...props} type={props.type ?? "button"} class={classes} {disabled}>
    {@render props.children?.()}
  </button>
{:else}
  <a {...props} class={classes}>
    {@render props.children?.()}
  </a>
{/if}

<style lang="postcss">
  @reference "../../../app";
  .glass-groupped-button {
    @pply relative;
    backdrop-filter: blur(0px);
  }

  .glass-groupped-button.active {
    @apply bg-white/20 text-white;
    backdrop-filter: blur(1px);
  }

  .glass-groupped-button::after {
    @apply absolute inset-0 w-full h-full opacity-0 pointer-events-none;
    @apply ring-2 ring-inset ring-primary-300;
    @apply transition-opacity duration-600 ease-in-out;
    content: "";
    border-radius: inherit;
    /* Ensure the shadow follows the button's border-radius */
  }

  .glass-groupped-button:hover::after {
    @apply opacity-100;
  }
</style>
