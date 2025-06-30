<script lang="ts">
  import type { SizeType } from "flowbite-svelte";
  import {
    type HTMLButtonAttributes,
    type HTMLAnchorAttributes,
  } from "svelte/elements";

  type BaseProps = {
    size?: SizeType;
    disabled?: boolean;
  };

  type Props =
    | (HTMLAnchorAttributes & BaseProps)
    | (HTMLButtonAttributes & BaseProps);

  const sizes: Record<SizeType, string> = {
    xs: "px-3 py-2 text-xs",
    sm: "px-4 py-2 text-sm",
    md: "px-5 py-2.5 text-sm",
    lg: "px-5 py-3 text-base",
    xl: "px-6 py-3.5 text-base",
  };

  let props: Props = $props();

  let { class: className, size = "md", disabled } = $derived(props);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  let classes = $derived([
    "glass-button rounded-3xl cursor-pointer",
    sizes[size],
    className,
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
  @reference "../../../app.css";

  .glass-button {
    @apply glassy-shadow relative;
    transition: box-shadow 1s ease-in-out;
  }

  .glass-button::after {
    content: "";
    opacity: 0;
    transition: opacity 0.5s ease-in-out;
    pointer-events: none;
    border-radius: inherit; /* Ensure the shadow follows the button's border-radius */
    @apply absolute inset-0 w-full h-full shadow-primary-300 shadow;
  }

  .glass-button:hover::after {
    opacity: 1;
  }
</style>
