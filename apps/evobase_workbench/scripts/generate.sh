#!/bin/bash

# Code generation script for EvoBase Workbench

echo "🔨 Generating code..."

# Clean previous builds
echo "📦 Cleaning..."
flutter clean
flutter pub get

# Generate code
echo "⚙️  Running build_runner..."
flutter pub run build_runner build --delete-conflicting-outputs

echo "✅ Code generation complete!"
echo ""
echo "Run the app with:"
echo "  flutter run -d macos"
echo "  flutter run -d windows"
echo "  flutter run -d linux"
