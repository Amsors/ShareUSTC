<template>
  <Navbar />
  <router-view />
  <PriorityModal />
</template>

<script setup lang="ts">
import Navbar from './components/Navbar.vue';
import PriorityModal from './components/notification/PriorityModal.vue';
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}
</style>