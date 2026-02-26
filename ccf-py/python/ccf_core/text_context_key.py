"""
TextContextKey — 5-dimensional SensorVocabulary for LLM conversation contexts.

Maps text conversation state into a quantised feature vector and stable FNV-1a hash.
Enables context-keyed trust accumulators without ML inference at tick time.

Invariants: I-LLM-001, I-LLM-008, I-LLM-009, I-LLM-010, I-LLM-011
Journey: J-LLM-FIRST-CONVERSATION
"""
from __future__ import annotations

import math
import time
from dataclasses import dataclass
from typing import Optional

# ── FNV-1a constants (must match ccf-core Rust implementation) ───────────────
_FNV_OFFSET_BASIS: int = 2166136261
_FNV_PRIME: int = 16777619


def _fnv1a(*values: int) -> int:
    """FNV-1a hash of a sequence of u32 values. 32-bit, matches Rust impl."""
    h = _FNV_OFFSET_BASIS
    for v in values:
        for byte_pos in range(4):
            byte = (v >> (byte_pos * 8)) & 0xFF
            h ^= byte
            h = (h * _FNV_PRIME) & 0xFFFFFFFF
    return h


# ── Dimension enumerations ────────────────────────────────────────────────────

class ConversationDepth:
    SHALLOW = 0
    MODERATE = 1
    DEEP = 2

    _LABELS = {0: "shallow", 1: "moderate", 2: "deep"}

    @classmethod
    def from_turn_count(cls, turn_count: int) -> int:
        if turn_count <= 3:
            return cls.SHALLOW
        if turn_count <= 15:
            return cls.MODERATE
        return cls.DEEP

    @classmethod
    def from_label(cls, label: str) -> int:
        mapping = {"shallow": 0, "moderate": 1, "deep": 2}
        result = mapping.get(label.lower())
        if result is None:
            raise ValueError(f"Unknown depth label: {label!r}")
        return result

    @classmethod
    def label(cls, v: int) -> str:
        return cls._LABELS.get(v, "moderate")


class EmotionalRegister:
    NEUTRAL = 0
    WARM = 1
    VULNERABLE = 2
    INTENSE = 3

    _LABELS = {0: "neutral", 1: "warm", 2: "vulnerable", 3: "intense"}

    # Keywords for simple heuristic detection
    _WARM_WORDS = frozenset({
        "love", "thank", "grateful", "appreciate", "wonderful", "happy", "joy",
        "friend", "family", "care", "kind", "warm", "nice",
    })
    _VULNERABLE_WORDS = frozenset({
        "anxiety", "afraid", "scared", "nervous", "worry",
        "struggle", "difficult", "hard", "overwhelm", "sad", "loss",
        "grief", "depressed", "lonely", "alone", "cry", "hurt",
    })
    _INTENSE_WORDS = frozenset({
        "urgent", "emergency", "crisis", "panic", "furious", "rage",
        "hate", "desperate", "critical", "immediately", "now", "must",
    })

    @classmethod
    def from_text(cls, text: str) -> int:
        words = set(text.lower().split())
        if words & cls._INTENSE_WORDS:
            return cls.INTENSE
        if words & cls._VULNERABLE_WORDS:
            return cls.VULNERABLE
        if words & cls._WARM_WORDS:
            return cls.WARM
        return cls.NEUTRAL

    @classmethod
    def from_label(cls, label: str) -> int:
        mapping = {"neutral": 0, "warm": 1, "vulnerable": 2, "intense": 3}
        result = mapping.get(label.lower())
        if result is None:
            raise ValueError(f"Unknown register label: {label!r}")
        return result

    @classmethod
    def label(cls, v: int) -> str:
        return cls._LABELS.get(v, "neutral")


class TimeOfDay:
    MORNING = 0    # 05:00–12:00
    AFTERNOON = 1  # 12:00–17:00
    EVENING = 2    # 17:00–21:00
    NIGHT = 3      # 21:00–05:00

    @classmethod
    def now(cls) -> int:
        hour = time.localtime().tm_hour
        if 5 <= hour < 12:
            return cls.MORNING
        if 12 <= hour < 17:
            return cls.AFTERNOON
        if 17 <= hour < 21:
            return cls.EVENING
        return cls.NIGHT

    @classmethod
    def from_hour(cls, hour: int) -> int:
        """Derive time-of-day from an explicit hour (0–23)."""
        if 5 <= hour < 12:
            return cls.MORNING
        if 12 <= hour < 17:
            return cls.AFTERNOON
        if 17 <= hour < 21:
            return cls.EVENING
        return cls.NIGHT

    @classmethod
    def label(cls, v: int) -> str:
        return {0: "morning", 1: "afternoon", 2: "evening", 3: "night"}.get(v, "afternoon")


class SessionPhase:
    OPENING = 0   # turns 0–3
    MIDDLE = 1    # turns 4–20
    CLOSING = 2   # turns 21+

    @classmethod
    def from_turn_count(cls, turn_count: int) -> int:
        if turn_count <= 3:
            return cls.OPENING
        if turn_count <= 20:
            return cls.MIDDLE
        return cls.CLOSING

    @classmethod
    def label(cls, v: int) -> str:
        return {0: "opening", 1: "middle", 2: "closing"}.get(v, "middle")


# ── Topic Domain (0–63 clusters) ─────────────────────────────────────────────

# Fallback keyword clusters when sentence-transformers is not available.
# 64 topic domains mapped from common keyword patterns.
_KEYWORD_CLUSTERS: list[tuple[frozenset, int]] = [
    # Finance & money (clusters 0–3)
    (frozenset({"finance", "money", "invest", "stock", "market", "budget", "saving", "wealth",
                "portfolio", "dividend", "compound", "interest", "bond", "fund",
                "personal", "financial"}), 0),
    (frozenset({"tax", "taxation", "irs", "deduction", "filing", "return", "income",
                "withhold"}), 1),
    (frozenset({"estate", "inheritance", "will", "trust", "beneficiary", "probate",
                "planning", "heir"}), 2),
    (frozenset({"crypto", "bitcoin", "ethereum", "blockchain", "defi", "nft", "token",
                "wallet", "cryptocurrency"}), 3),
    # Health & wellness (clusters 4–7)
    (frozenset({"health", "exercise", "fitness", "gym", "workout", "diet", "nutrition",
                "weight", "run", "yoga", "meditat"}), 4),
    (frozenset({"medical", "doctor", "symptom", "diagnosis", "treatment", "medication",
                "hospital", "surgery", "therapy"}), 5),
    (frozenset({"mental", "anxiety", "depression", "stress", "counseling",
                "psychology", "mindful", "emotional", "wellbeing"}), 6),
    (frozenset({"sleep", "insomnia", "rest", "fatigue", "tired", "nap", "circadian"}), 7),
    # Technology (clusters 8–11)
    (frozenset({"code", "programming", "software", "developer", "python", "javascript",
                "rust", "algorithm", "debug", "git"}), 8),
    (frozenset({"ai", "artificial", "intelligence", "machine", "learning", "neural",
                "model", "llm", "gpt", "claude"}), 9),
    (frozenset({"robot", "autonomous", "sensor", "embedded", "hardware",
                "microcontroller"}), 10),
    (frozenset({"cloud", "aws", "azure", "gcp", "kubernetes", "docker", "devops",
                "deploy"}), 11),
    # Relationships & social (clusters 12–15)
    (frozenset({"relationship", "partner", "dating", "marriage", "divorce", "breakup",
                "love", "romance", "attraction"}), 12),
    (frozenset({"family", "parent", "child", "sibling", "mother", "father",
                "children"}), 13),
    (frozenset({"friend", "friendship", "social", "community", "networking",
                "connection"}), 14),
    (frozenset({"conflict", "argument", "disagree", "boundary", "communication",
                "assertive"}), 15),
    # Career & work (clusters 16–19)
    (frozenset({"career", "job", "work", "resume", "interview", "salary", "promotion",
                "workplace", "professional"}), 16),
    (frozenset({"startup", "entrepreneur", "business", "venture", "founder",
                "company"}), 17),
    (frozenset({"management", "leadership", "team", "meeting", "productivity",
                "agile"}), 18),
    (frozenset({"creative", "design", "art", "write", "writing", "author", "paint",
                "draw"}), 19),
    # Education (clusters 20–23)
    (frozenset({"study", "learn", "course", "university", "school", "education",
                "degree"}), 20),
    (frozenset({"science", "research", "experiment", "hypothesis", "data",
                "analysis"}), 21),
    (frozenset({"math", "calculus", "algebra", "statistic", "probability",
                "number"}), 22),
    (frozenset({"history", "historical", "ancient", "civilization", "war",
                "empire"}), 23),
    # Lifestyle (clusters 24–27)
    (frozenset({"food", "cook", "recipe", "restaurant", "eat", "cuisine", "meal"}), 24),
    (frozenset({"travel", "trip", "vacation", "flight", "hotel", "country",
                "abroad"}), 25),
    (frozenset({"home", "house", "interior", "renovate", "garden", "decor",
                "furniture"}), 26),
    (frozenset({"hobby", "game", "sport", "music", "movie", "book",
                "entertainment"}), 27),
    # Philosophy & values (clusters 28–31)
    (frozenset({"philosophy", "ethics", "moral", "value", "meaning", "purpose",
                "existential"}), 28),
    (frozenset({"religion", "spiritual", "faith", "belief", "prayer", "god",
                "universe"}), 29),
    (frozenset({"politic", "government", "policy", "vote", "democracy",
                "election"}), 30),
    (frozenset({"environment", "climate", "sustainable", "eco", "green",
                "carbon"}), 31),
]


def _keyword_cluster(text: str) -> int:
    """
    Map text to a topic cluster using keyword matching.

    Returns cluster 32 ("miscellaneous") if no keywords match.
    """
    # Normalise: lowercase, strip punctuation
    tokens = set(
        word.strip(".,?!;:'\"") for word in text.lower().split()
    )
    best_cluster = 32  # miscellaneous
    best_score = 0
    for keywords, cluster in _KEYWORD_CLUSTERS:
        score = len(tokens & keywords)
        if score > best_score:
            best_score = score
            best_cluster = cluster
    return best_cluster


def _sentence_transformer_cluster(text: str) -> int:
    """
    Map text to cluster 0–63 using sentence-transformers if available.

    Falls back to keyword matching if sentence-transformers is not installed
    or centroids have not been pre-computed.
    """
    try:
        from sentence_transformers import SentenceTransformer  # type: ignore
        import numpy as np  # type: ignore

        # Lazy-load model (cached after first call via function attribute)
        if not hasattr(_sentence_transformer_cluster, "_model"):
            _sentence_transformer_cluster._model = SentenceTransformer(
                "all-MiniLM-L6-v2"
            )
            _sentence_transformer_cluster._centroids = None  # pre-compute offline

        if _sentence_transformer_cluster._centroids is not None:
            embedding = _sentence_transformer_cluster._model.encode(text)
            centroids = _sentence_transformer_cluster._centroids
            norms = np.linalg.norm(centroids, axis=1) * np.linalg.norm(embedding)
            sims = np.dot(centroids, embedding) / (norms + 1e-9)
            return int(np.argmax(sims))
    except ImportError:
        pass
    return _keyword_cluster(text)


# ── TextContextKey ────────────────────────────────────────────────────────────

@dataclass(frozen=True)
class TextContextKey:
    """
    5-dimensional SensorVocabulary for LLM conversation contexts.

    Produces a stable FNV-1a hash and normalised feature vector from quantised
    conversation dimensions. Enables context-keyed trust accumulators.

    Dimensions
    ----------
    topic_domain : int
        0–63 topic cluster derived from text.
    conversation_depth : int
        0=shallow, 1=moderate, 2=deep.
    emotional_register : int
        0=neutral, 1=warm, 2=vulnerable, 3=intense.
    time_of_day : int
        0=morning(5–12), 1=afternoon(12–17), 2=evening(17–21), 3=night(21–5).
    session_phase : int
        0=opening(turns 0–3), 1=middle(turns 4–20), 2=closing(turns 21+).

    Invariants
    ----------
    I-LLM-001 : Identical inputs always produce the same hash (FNV-1a).
    I-LLM-008 : topic_domain is always in 0–63.
    I-LLM-010 : Pure Python — no Rust FFI required.
    I-LLM-011 : Full key derivation < 50 ms.

    Usage
    -----
    >>> key = TextContextKey.from_text("What is compound interest?")
    >>> key.context_hash()        # stable FNV-1a u32
    >>> key.feature_vector()      # 5 floats in [0, 1]
    """

    topic_domain: int       # 0–63 embedding cluster
    conversation_depth: int    # 0=shallow, 1=moderate, 2=deep
    emotional_register: int    # 0=neutral, 1=warm, 2=vulnerable, 3=intense
    time_of_day: int           # 0=morning, 1=afternoon, 2=evening, 3=night
    session_phase: int         # 0=opening, 1=middle, 2=closing

    def __post_init__(self) -> None:
        if not (0 <= self.topic_domain <= 63):
            raise ValueError(
                f"topic_domain must be 0–63, got {self.topic_domain}"
            )
        if not (0 <= self.conversation_depth <= 2):
            raise ValueError(
                f"conversation_depth must be 0–2, got {self.conversation_depth}"
            )
        if not (0 <= self.emotional_register <= 3):
            raise ValueError(
                f"emotional_register must be 0–3, got {self.emotional_register}"
            )
        if not (0 <= self.time_of_day <= 3):
            raise ValueError(
                f"time_of_day must be 0–3, got {self.time_of_day}"
            )
        if not (0 <= self.session_phase <= 2):
            raise ValueError(
                f"session_phase must be 0–2, got {self.session_phase}"
            )

    # ── Core API ─────────────────────────────────────────────────────────────

    def context_hash(self) -> int:
        """
        FNV-1a hash of the 5-tuple.

        Stable across Python versions, runs, and machines.
        Matches the Rust ccf-core implementation (I-LLM-001).
        """
        return _fnv1a(
            self.topic_domain,
            self.conversation_depth,
            self.emotional_register,
            self.time_of_day,
            self.session_phase,
        )

    def feature_vector(self) -> list[float]:
        """
        Normalised float vector in [0, 1]^5.

        Layout: [topic/63, depth/2, register/3, time/3, phase/2]
        """
        return [
            self.topic_domain / 63.0,
            self.conversation_depth / 2.0,
            self.emotional_register / 3.0,
            self.time_of_day / 3.0,
            self.session_phase / 2.0,
        ]

    def cosine_similarity(self, other: "TextContextKey") -> float:
        """
        Cosine similarity between two feature vectors.

        Returns a value in [0, 1]. Returns 0.0 if either vector is zero.
        """
        a = self.feature_vector()
        b = other.feature_vector()
        dot = sum(x * y for x, y in zip(a, b))
        norm_a = math.sqrt(sum(x * x for x in a))
        norm_b = math.sqrt(sum(x * x for x in b))
        if norm_a < 1e-9 or norm_b < 1e-9:
            return 0.0
        return dot / (norm_a * norm_b)

    def label(self) -> str:
        """Human-readable label for logging and dashboard display."""
        return (
            f"topic:{self.topic_domain}"
            f":{ConversationDepth.label(self.conversation_depth)}"
            f":{EmotionalRegister.label(self.emotional_register)}"
            f":{TimeOfDay.label(self.time_of_day)}"
            f":{SessionPhase.label(self.session_phase)}"
        )

    # ── Factory methods ───────────────────────────────────────────────────────

    @classmethod
    def from_text(
        cls,
        text: str,
        depth: "str | int" = "moderate",
        register: "Optional[str | int]" = None,
        turn_count: int = 0,
        use_embeddings: bool = False,
    ) -> "TextContextKey":
        """
        Derive a TextContextKey from conversation text.

        Parameters
        ----------
        text : str
            The user message or topic description.
        depth : str or int, optional
            Conversation depth — "shallow"|"moderate"|"deep" or int 0–2.
            If not provided, inferred from turn_count.
        register : str or int or None, optional
            Emotional register — "neutral"|"warm"|"vulnerable"|"intense" or
            int 0–3. If None, inferred from text keywords.
        turn_count : int, optional
            Current turn number (used to infer depth and session_phase).
        use_embeddings : bool, optional
            Use sentence-transformers for topic clustering (slow, accurate).
            Defaults to False (fast keyword matching, no ML deps required).

        Returns
        -------
        TextContextKey
            Fully populated key ready to hash and vectorise.
        """
        # Resolve topic domain
        if use_embeddings:
            topic = _sentence_transformer_cluster(text)
        else:
            topic = _keyword_cluster(text)

        # Resolve depth
        if isinstance(depth, str):
            depth_int = ConversationDepth.from_label(depth)
        elif isinstance(depth, int):
            depth_int = depth
        else:
            depth_int = ConversationDepth.from_turn_count(turn_count)

        # Resolve register
        if register is None:
            register_int = EmotionalRegister.from_text(text)
        elif isinstance(register, str):
            register_int = EmotionalRegister.from_label(register)
        else:
            register_int = int(register)

        return cls(
            topic_domain=topic,
            conversation_depth=depth_int,
            emotional_register=register_int,
            time_of_day=TimeOfDay.now(),
            session_phase=SessionPhase.from_turn_count(turn_count),
        )

    @classmethod
    def nearest_contexts(
        cls,
        candidate: "TextContextKey",
        known: "list[TextContextKey]",
        k: int = 3,
    ) -> "list[tuple[TextContextKey, float]]":
        """
        Return k nearest known contexts by cosine similarity (warm-start seeding).

        Parameters
        ----------
        candidate : TextContextKey
            The new, unseen context to seed.
        known : list of TextContextKey
            Contexts with accumulated trust history.
        k : int, optional
            Maximum number of neighbours to return. Default 3.

        Returns
        -------
        list of (TextContextKey, float)
            Sorted by similarity descending.
        """
        if not known:
            return []
        scored = [(ctx, candidate.cosine_similarity(ctx)) for ctx in known]
        scored.sort(key=lambda pair: pair[1], reverse=True)
        return scored[:k]
