import { Capacitor } from '@capacitor/core';
import { PushNotifications } from '@capacitor/push-notifications';
import { App } from '@capacitor/app';
import { ScreenOrientation } from '@capacitor/screen-orientation';
import { StatusBar, Style } from '@capacitor/status-bar';
import { apiFetch } from '../api/client';

export async function initCapacitorApp(): Promise<void> {
  if (!Capacitor.isNativePlatform()) return;

  try {
    await StatusBar.setStyle({ style: Style.Dark });
    await StatusBar.setBackgroundColor({ color: '#0a0a0c' });
  } catch (e) {
    console.warn('StatusBar error:', e);
  }

  // Handle hardware back button
  App.addListener('backButton', ({ canGoBack }) => {
    if (window.location.hash === '#/gallery' || window.location.hash === '' || window.location.hash === '#/') {
      App.exitApp();
    } else if (canGoBack) {
      window.history.back();
    } else {
      window.location.hash = '#/gallery';
    }
  });
}

export async function initPushNotifications(): Promise<void> {
  if (!Capacitor.isNativePlatform()) return;

  try {
    let permStatus = await PushNotifications.checkPermissions();

    if (permStatus.receive === 'prompt') {
      permStatus = await PushNotifications.requestPermissions();
    }

    if (permStatus.receive !== 'granted') {
      console.log('Push notification permission denied');
      return;
    }

    await PushNotifications.register();

    await PushNotifications.addListener('registration', async (token) => {
      console.log('FCM Token registered:', token.value);
      try {
        await apiFetch('/users/me/devices', {
          method: 'POST',
          body: JSON.stringify({ fcm_token: token.value, platform: 'android' }),
        });
      } catch (err) {
        console.error('Failed to send FCM token to backend:', err);
      }
    });

    await PushNotifications.addListener('registrationError', (error) => {
      console.error('Push registration error:', error);
    });

    await PushNotifications.addListener('pushNotificationReceived', (notification) => {
      console.log('Push notification received:', notification);
    });

    await PushNotifications.addListener('pushNotificationActionPerformed', (notificationAction) => {
      console.log('Push notification action performed:', notificationAction);
      const data = notificationAction.notification.data;
      if (data && data.video_id) {
        window.location.hash = `#/player/${data.video_id}`;
      }
    });
  } catch (e) {
    console.warn('Push notification error:', e);
  }
}

export async function setLandscapeOrientation(landscape: boolean): Promise<void> {
  if (!Capacitor.isNativePlatform()) return;
  try {
    if (landscape) {
      await ScreenOrientation.lock({ orientation: 'landscape' });
    } else {
      await ScreenOrientation.unlock();
    }
  } catch (e) {
    console.warn('ScreenOrientation error:', e);
  }
}
