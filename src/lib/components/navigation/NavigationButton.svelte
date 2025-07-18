<script lang="ts">
  import { getContext, type Snippet } from "svelte";
  import {
    type HTMLButtonAttributes,
    type HTMLAnchorAttributes,
  } from "svelte/elements";

  type BaseProps = {
    active?: boolean;
    disabled?: boolean;
    icon: Snippet<[string]>;
    label: Snippet;
  };

  type Props =
    | (HTMLAnchorAttributes & BaseProps)
    | (HTMLButtonAttributes & BaseProps);

  let props: Props = $props();
  let { icon, label } = $derived(props);
  const { disabled: ctxDisabled } =
    getContext<() => { disabled: boolean }>("navigation")();

  let { disabled: propDisabled, active } = $derived(props);
  let disabled = $derived(ctxDisabled || propDisabled);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  let classes = $derived([
    "navigation-button",
    "flex flex-col gap-1",
    "transition-all duration-500",
    "items-center text-sm",
    "rounded-4xl cursor-pointer",
    "py-1.5",
    active ? "px-10" : "px-5",
    { active },
  ]);
</script>

{#snippet inner()}
  <div>
    {@render icon("h-5 w-5")}
  </div>
  <div>
    {@render label()}
  </div>
{/snippet}

{#if isButtonProps(props)}
  <button {...props} type={props.type ?? "button"} class={classes} {disabled}>
    {@render inner()}
  </button>
{:else}
  <a {...props} class={classes}>
    {@render inner()}
  </a>
{/if}

<style lang="postcss">
  @reference "../../../app";

  .navigation-button {
    @apply text-white/60;
    @apply relative;
  }
  .navigation-button.active {
    @apply text-white;
  }

  .navigation-button::after {
    @apply absolute inset-0 w-full h-full opacity-0 pointer-events-none;
    @apply bg-white/20;
    z-index: -1;
    opacity: 0;
    @apply transition-all duration-500;
    backdrop-filter: blur(1px);
    content: "";
    border-radius: inherit;
  }

  .navigation-button.active::after {
    opacity: 100;
  }
</style>
