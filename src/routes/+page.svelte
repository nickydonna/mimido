<script lang="ts">
import Register from "flowbite-svelte-blocks/Register.svelte";
import Section from 'flowbite-svelte-blocks/Section.svelte';
import { Button, Checkbox, Label, Input } from "flowbite-svelte";
</script>

<Section name="login">
  <Register href="/">
    <svelte:fragment slot="top">
      MimiDo
    </svelte:fragment>
    <div class="p-6 space-y-4 md:space-y-6 sm:p-8">
      <form class="flex flex-col space-y-6" action="/">
        <h3 class="text-xl font-medium text-gray-900 dark:text-white p-0">Login</h3>
        <Label class="space-y-2">
          <span>Your email</span>
          <Input type="email" name="email" placeholder="name@company.com" required />
        </Label>
        <Label class="space-y-2">
          <span>Your password</span>
          <Input type="password" name="password" placeholder="•••••" required />
        </Label>
        <Button type="submit" class="w-full1">Sign in</Button>
      </form>
    </div>
  </Register>
</Section>

