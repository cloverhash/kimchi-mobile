# Expo Example App

This example demonstrates using the Kimchi Expo module in a React Native/Expo app.

## Setup

1. Create a new Expo app (or use existing):
   ```bash
   npx create-expo-app@latest KimchiDemo
   cd KimchiDemo
   ```

2. Install the Kimchi Expo module:
   ```bash
   # From local path (not yet published to npm)
   npm install ../../packages/expo
   ```

3. Run prebuild (required for native modules):
   ```bash
   npx expo prebuild
   ```

4. Use in your app:
   ```tsx
   import { useState } from 'react';
   import { Button, Text, View, ActivityIndicator } from 'react-native';
   import {
     initialize,
     proveThreshold,
     verifyProof,
     freeProof,
     createShareableProof
   } from '@kimchi/expo';

   export default function App() {
     const [status, setStatus] = useState('');
     const [loading, setLoading] = useState(false);

     const runDemo = async () => {
       setLoading(true);
       try {
         // Initialize the prover (do this once at app startup)
         setStatus('Initializing...');
         await initialize(12); // Use smaller SRS for testing

         // Generate a proof that 42 < 100
         setStatus('Generating proof...');
         const proof = await proveThreshold(42n, 100n);
         setStatus(`Proof generated in ${proof.generationTimeMs}ms`);

         // Verify the proof locally
         const isValid = await verifyProof(proof.proofHandle);
         setStatus(prev => prev + `\nProof valid: ${isValid}`);

         // Create shareable proof for remote verification
         const shareable = await createShareableProof(proof);
         console.log('Shareable proof:', JSON.stringify(shareable));

         // Free the proof when done
         await freeProof(proof.proofHandle);
         setStatus(prev => prev + '\nProof freed');
       } catch (error) {
         setStatus(`Error: ${error.message}`);
       } finally {
         setLoading(false);
       }
     };

     return (
       <View style={{ flex: 1, justifyContent: 'center', padding: 20 }}>
         <Text style={{ marginBottom: 20, textAlign: 'center' }}>
           {status || 'Press button to generate proof'}
         </Text>
         {loading ? (
           <ActivityIndicator size="large" />
         ) : (
           <Button title="Generate Proof" onPress={runDemo} />
         )}
       </View>
     );
   }
   ```

5. Run the app:
   ```bash
   # iOS
   npx expo run:ios

   # Android
   npx expo run:android
   ```

## Key Points

- **Cannot run in Expo Go** - requires `expo prebuild` due to native binary
- Uses `bigint` for value/threshold to handle 64-bit integers
- Proof generation is CPU-intensive; always show loading indicator
- Call `freeProof()` when done to release memory
- Use smaller SRS sizes (10-12) during development for faster testing

## Remote Verification

The generated shareable proof can be verified in a browser or Node.js using the WASM verifier:

```typescript
import { initVerifier, verifyProof } from '@kimchi/wasm';

// Initialize the verifier (once)
await initVerifier(12);

// Verify the proof
const isValid = verifyProof(
  shareable.proof,
  shareable.verifierIndex,
  shareable.publicInputs
);
```
