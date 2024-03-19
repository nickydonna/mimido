<script>
	import '../app.pcss';

  import {  RectangleListOutline,  CalendarEditOutline, PlusOutline } from 'flowbite-svelte-icons';
	import BottomNav from 'flowbite-svelte/BottomNav.svelte';
	import BottomNavItem from 'flowbite-svelte/BottomNavItem.svelte';
	import Navbar from 'flowbite-svelte/Navbar.svelte';
	import NavBrand from 'flowbite-svelte/NavBrand.svelte';
	import NavLi from 'flowbite-svelte/NavLi.svelte';
	import NavUl from 'flowbite-svelte/NavUl.svelte';
	import NavHamburger from 'flowbite-svelte/NavHamburger.svelte';
</script>

<div class="container mx-auto">
	<Navbar>
		<NavBrand href="/">
			<span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">MimiDo</span>
		</NavBrand>
  	<NavHamburger  />
  	<NavUl >
  	  <NavLi href="/">Home</NavLi>
   	 <NavLi href="/about">About</NavLi>
    	<NavLi href="/docs/components/navbar">Navbar</NavLi>
    	<NavLi href="/pricing">Pricing</NavLi>
    	<NavLi href="/contact">Contact</NavLi>
  	</NavUl>
	</Navbar>
	<slot />	
	<BottomNav position="fixed" classInner="grid-cols-3">
		<BottomNavItem btnName="ListTask">
			<RectangleListOutline class="w-5 h-5 mb-1 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500" />
		</BottomNavItem>
		<BottomNavItem btnName="CalenderView" href="/day">
			<CalendarEditOutline class="w-5 h-5 mb-1 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500" />
		</BottomNavItem>
		<BottomNavItem btnName="Add" href="/form">
			<PlusOutline class="w-5 h-5 mb-1 text-gray-500 dark:text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-500" />
		</BottomNavItem>
	</BottomNav>
</div>
