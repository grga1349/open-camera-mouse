import {useState} from 'react';
import {MainScreen} from './screens/MainScreen';
import {SettingsScreen} from './screens/SettingsScreen';
import {AppProvider} from './state/useAppStore';

type Screen = 'main' | 'settings';

function App() {
	const [screen, setScreen] = useState<Screen>('main');

	const openSettings = () => setScreen('settings');
	const closeSettings = () => setScreen('main');

return (
	<AppProvider>
		{screen === 'main' ? (
			<MainScreen onOpenSettings={openSettings}/>
		) : (
			<SettingsScreen onCancel={closeSettings} onSave={closeSettings}/>
		)}
	</AppProvider>
);
}

export default App;
