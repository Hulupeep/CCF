/**
 * mBot Mobile App - Main Entry Point
 * Issue: #88 (STORY-MOBILE-001)
 */

import React from 'react';
import { StatusBar } from 'expo-status-bar';
import { NavigationContainer } from '@react-navigation/native';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import Icon from 'react-native-vector-icons/Ionicons';

import { DiscoveryScreen } from './screens/DiscoveryScreen';
import { PersonalityMixerScreen } from './screens/PersonalityMixerScreen';
import { NeuralVisualizerScreen } from './screens/NeuralVisualizerScreen';
import { GalleryScreen } from './screens/GalleryScreen';

const Tab = createBottomTabNavigator();

export default function App() {
  return (
    <NavigationContainer>
      <StatusBar style="auto" />

      <Tab.Navigator
        screenOptions={({ route }) => ({
          tabBarIcon: ({ focused, color, size }) => {
            let iconName: string = 'help-circle';

            if (route.name === 'Discovery') {
              iconName = focused ? 'search' : 'search-outline';
            } else if (route.name === 'Mixer') {
              iconName = focused ? 'options' : 'options-outline';
            } else if (route.name === 'Neural') {
              iconName = focused ? 'git-network' : 'git-network-outline';
            } else if (route.name === 'Gallery') {
              iconName = focused ? 'images' : 'images-outline';
            }

            return <Icon name={iconName} size={size} color={color} />;
          },
          tabBarActiveTintColor: '#007AFF',
          tabBarInactiveTintColor: 'gray',
          headerShown: false,
        })}
      >
        <Tab.Screen
          name="Discovery"
          component={DiscoveryScreen}
          options={{ tabBarLabel: 'Connect' }}
        />
        <Tab.Screen
          name="Mixer"
          component={PersonalityMixerScreen}
          options={{ tabBarLabel: 'Personality' }}
        />
        <Tab.Screen
          name="Neural"
          component={NeuralVisualizerScreen}
          options={{ tabBarLabel: 'Neural' }}
        />
        <Tab.Screen
          name="Gallery"
          component={GalleryScreen}
          options={{ tabBarLabel: 'Gallery' }}
        />
      </Tab.Navigator>
    </NavigationContainer>
  );
}
