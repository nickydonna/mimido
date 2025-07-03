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
    xs: "p-2 text-xs",
    sm: "p-3  text-sm",
    md: "p-4  text-sm",
    lg: "p-5 text-base",
    xl: "p-6 text-lg",
  };

  let props: Props = $props();

  let { class: className, size = "md", disabled } = $derived(props);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  let classes = $derived([
    "glass-clickable rounded-3xl cursor-pointer",
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
    transition: opacity 1s ease-in-out;
    pointer-events: none;
    border-radius: inherit; /* Ensure the shadow follows the button's border-radius */
    @apply absolute inset-0 w-full h-full inset-ring-primary-300 inset-ring-2;
  }

  .glass-button:hover::after {
    opacity: 1;
  }
</style>
