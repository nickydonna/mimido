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
    xs: "p-1 text-xs",
    sm: "p-2 text-sm",
    md: "p-3 text-sm",
    lg: "p-4 text-base",
    xl: "p-5 text-lg",
  };

  let props: Props = $props();

  let { class: className, size = "md", disabled } = $derived(props);

  function isButtonProps(
    props: Props,
  ): props is HTMLButtonAttributes & BaseProps {
    return !("href" in props);
  }

  let classes = $derived([
    "glass-clickable rounded-4xl cursor-pointer",
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
