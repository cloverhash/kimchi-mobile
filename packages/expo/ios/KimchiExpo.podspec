require 'json'

package = JSON.parse(File.read(File.join(__dir__, '..', 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'KimchiExpo'
  s.version        = package['version']
  s.summary        = package['description']
  s.description    = package['description']
  s.license        = package['license']
  s.author         = package['author']
  s.homepage       = package['homepage']
  s.platforms      = { :ios => '15.0' }
  s.swift_version  = '5.4'
  s.source         = { :git => 'https://github.com/user/kimchi-mobile.git' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  # Option 1: If KimchiMobile is published as a CocoaPod
  # s.dependency 'KimchiMobile', '~> 1.0'

  # Option 2: Use the XCFramework directly
  # The XCFramework path is relative to this podspec
  s.vendored_frameworks = '../../../ios-output/KimchiFfi.xcframework'

  # Swift source files for the Expo module
  s.source_files = '*.swift'

  # Link with the Swift package
  # Note: Consumer must also add KimchiMobile SPM package or vendored framework

  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule'
  }
end
