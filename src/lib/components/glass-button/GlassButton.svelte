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
