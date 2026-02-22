#[derive(Debug, Clone)]
pub enum Flag {
    OptimizationLevel(u8),
    OptimizeSize,
    OptimizeSpeed,
    NoOptimization,
    DependencyInfo(String),

    DebugInfo,
    DebugInfoFull,
    NoDebugInfo,

    AllWarnings,
    WarningsAsErrors,
    ExtraWarnings,
    NoWarnings,
    SpecificWarning(String),

    CxxStandard(String),
    CStandard(String),

    IncludePath(String),
    SystemIncludePath(String),

    Define(String, Option<String>),

    CompileOnly,
    OutputFile(String),
    ObjectOutput(String),

    LinkLibrary(String),
    LibraryPath(String),
    StaticLink,
    SharedLink,

    PositionIndependentCode,
    LinkTimeOptimization,
    NoLinkTimeOptimization,
    ThinLTO,
    FatLTO,

    TargetArch(String),

    NoLogo,
    MultiThreadedDLL,
    MultiThreadedDLLDebug,
    ExceptionHandling,

    MachineDependent(String),
    FeatureFlag(String),

    ShowIncludes,
    SourceDependencies(String),

    ProgramDatabase(String),
    LinkProgramDatabase(String),
    DebugType(String),

    Raw(String),
}
