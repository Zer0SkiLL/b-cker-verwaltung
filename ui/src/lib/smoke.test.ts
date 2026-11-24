import { render, screen } from '@testing-library/svelte';
import App from '../App.svelte';

test('App zeigt den Programmnamen', () => {
  render(App);
  expect(screen.getByRole('heading', { name: 'Mitgliederverwaltung' })).toBeInTheDocument();
});
