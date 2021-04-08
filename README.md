# wasm-stack-consumer

A simple binary analyzer for stack allocation size in WebAssembly based on LLVM code generation.

## Usage

First, you need to prepare a list of function names you want to know the stack alloc size.

```
$ cat callstack.log
swift::SubstGenericParametersFromMetadata::setup() const
swift::TargetMetadata<swift::InProcess> const* std::__2::__function::__policy_invoker<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>::__call_impl<std::__2::__function::__default_alloc_func<swift_getTypeByMangledNameInContext::$_5, swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)> >(std::__2::__function::__policy_storage const*, unsigned int, unsigned int)
swift::Demangle::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::Node*)
swift::Demangle::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::Node*)
swift::Demangle::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::Node*)
swift::Demangle::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::Node*)
swift_getTypeByMangledNodeImpl(swift::MetadataRequest, swift::Demangle::Demangler&, swift::Demangle::Node*, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
swift::swift_getTypeByMangledNode(swift::MetadataRequest, swift::Demangle::Demangler&, swift::Demangle::Node*, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
swift_getTypeByMangledNameImpl(swift::MetadataRequest, llvm::StringRef, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
swift::swift_getTypeByMangledName(swift::MetadataRequest, llvm::StringRef, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
swift_getTypeByMangledNameInContext
$sSP11TokamakCoreAA11FieldRecordVRszlE4type14genericContext0F9ArgumentsypXpSVSg_AGtF
...
```

Then, this analyzer can tell you the size of stack allocation for each function

```
$ cargo run -- main.wasm callstack.log
func[62140] size = 304 swift::SubstGenericParametersFromMetadata::buildDescriptorPath(swift::TargetContextDescriptor<swift::InProcess> const*, swift::Demangle::__runtime::Demangler&) const
func[62140] size = 304 swift::SubstGenericParametersFromMetadata::buildDescriptorPath(swift::TargetContextDescriptor<swift::InProcess> const*, swift::Demangle::__runtime::Demangler&) const
func[62140] size = 304 swift::SubstGenericParametersFromMetadata::buildDescriptorPath(swift::TargetContextDescriptor<swift::InProcess> const*, swift::Demangle::__runtime::Demangler&) const
func[62142] size = 2352 swift::SubstGenericParametersFromMetadata::setup() const
couldn't estimate stack size swift::TargetMetadata<swift::InProcess> const* std::__2::__function::__policy_invoker<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>::__call_impl<std::__2::__function::__default_alloc_func<swift_getTypeByMangledNameInContext::$_5, swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)> >(std::__2::__function::__policy_storage const*, unsigned int, unsigned int) (not produced by LLVM: missing global.get but got LocalGet { local_index: 0 }
func[62162] size = 736 swift::Demangle::__runtime::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::__runtime::Node*)
func[62162] size = 736 swift::Demangle::__runtime::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::__runtime::Node*)
func[62162] size = 736 swift::Demangle::__runtime::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::__runtime::Node*)
func[62162] size = 736 swift::Demangle::__runtime::TypeDecoder<(anonymous namespace)::DecodedMetadataBuilder>::decodeMangledType(swift::Demangle::__runtime::Node*)
func[62160] size = 80 swift_getTypeByMangledNodeImpl(swift::MetadataRequest, swift::Demangle::__runtime::Demangler&, swift::Demangle::__runtime::Node*, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
func[62158] size = 64 swift::swift_getTypeByMangledNode(swift::MetadataRequest, swift::Demangle::__runtime::Demangler&, swift::Demangle::__runtime::Node*, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
func[62123] size = 2432 swift_getTypeByMangledNameImpl(swift::MetadataRequest, __swift::__runtime::llvm::StringRef, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
func[62121] size = 64 swift::swift_getTypeByMangledName(swift::MetadataRequest, __swift::__runtime::llvm::StringRef, void const* const*, std::__2::function<swift::TargetMetadata<swift::InProcess> const* (unsigned int, unsigned int)>, std::__2::function<swift::TargetWitnessTable<swift::InProcess> const* (swift::TargetMetadata<swift::InProcess> const*, unsigned int)>)
func[62127] size = 272 swift_getTypeByMangledNameInContext
func[32872] size = 16 $sSP11TokamakCoreAA11FieldRecordVRszlE4type14genericContext0F9ArgumentsypXpSVSg_AGtF
...
Total size: 56224
```
