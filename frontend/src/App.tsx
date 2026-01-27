import {useState} from 'react';
import {MainScreen} from './screens/MainScreen';
import {SettingsScreen} from './screens/SettingsScreen';

type Screen = 'main' | 'settings';

function App() {
	const [screen, setScreen] = useState<Screen>('main');

	const openSettings = () => setScreen('settings');
	const closeSettings = () => setScreen('main');

	return screen === 'main' ? (
		<MainScreen onOpenSettings={openSettings}/>
	) : (
		<SettingsScreen onCancel={closeSettings} onSave={closeSettings}/>
	);
}

export default App;
