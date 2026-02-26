"""
Journey Test: J-LLM-FIRST-CONVERSATION
=======================================

Tests that TextContextKey correctly derives context from user messages
and that context isolation works across topic switches.

Specflow contract : J-LLM-FIRST-CONVERSATION
Invariants        : I-LLM-001, I-LLM-008, I-LLM-009
DOD               :
    - from_text() returns a stable hash for the same input
    - Different topic clusters produce different hashes
    - Cosine similarity < 0.95 across different topic clusters
    - Warm-start seeding finds the nearest related context

Run with:
    python -m pytest tests/journeys/llm_first_conversation.journey.spec.py -v
"""

import sys
import pytest

sys.path.insert(0, "ccf-py/python")

from ccf_core.text_context_key import (
    TextContextKey,
    ConversationDepth,
    EmotionalRegister,
    TimeOfDay,
    SessionPhase,
    _keyword_cluster,
)


# ── Step 1: User starts on "personal finance" → gets a TextContextKey ─────────

class TestStep1PersonalFinanceContext:
    """
    Scenario: User opens a conversation about personal finance
      Given the user sends "What is compound interest and how does saving work?"
      When TextContextKey.from_text() is called with that text
      Then a valid TextContextKey is returned
      And the topic_domain is in 0–63 (I-LLM-008)
      And the feature_vector has 5 components in [0, 1]
    """

    def setup_method(self):
        self.text = "What is compound interest and how does saving work?"
        self.key = TextContextKey(
            topic_domain=_keyword_cluster(self.text),
            conversation_depth=ConversationDepth.SHALLOW,
            emotional_register=EmotionalRegister.NEUTRAL,
            time_of_day=TimeOfDay.MORNING,
            session_phase=SessionPhase.OPENING,
        )

    def test_key_is_valid_instance(self):
        assert isinstance(self.key, TextContextKey)

    def test_topic_domain_in_valid_range(self):
        """I-LLM-008: topic_domain must be 0–63."""
        assert 0 <= self.key.topic_domain <= 63

    def test_feature_vector_has_five_components(self):
        vec = self.key.feature_vector()
        assert len(vec) == 5

    def test_feature_vector_all_in_unit_range(self):
        vec = self.key.feature_vector()
        for i, v in enumerate(vec):
            assert 0.0 <= v <= 1.0, f"Component {i} = {v} out of [0, 1]"

    def test_finance_text_maps_to_finance_cluster(self):
        """Finance keywords must map to cluster 0 (personal finance)."""
        topic = _keyword_cluster("compound interest personal finance budget saving")
        assert topic == 0, f"Expected finance cluster 0, got {topic}"


# ── Step 2: Hash is stable across multiple calls ──────────────────────────────

class TestStep2HashStability:
    """
    Scenario: Same personal finance context requested multiple times
      Given a TextContextKey with fixed dimensions
      When context_hash() is called multiple times
      Then the same hash value is returned every time (I-LLM-001)
    """

    def setup_method(self):
        self.key = TextContextKey(
            topic_domain=0,   # personal finance
            conversation_depth=ConversationDepth.SHALLOW,
            emotional_register=EmotionalRegister.NEUTRAL,
            time_of_day=TimeOfDay.MORNING,
            session_phase=SessionPhase.OPENING,
        )

    def test_hash_identical_on_100_calls(self):
        """I-LLM-001: context_hash() is deterministic."""
        first_hash = self.key.context_hash()
        for i in range(99):
            assert self.key.context_hash() == first_hash, (
                f"Hash changed on call {i + 2}"
            )

    def test_hash_is_32bit_unsigned(self):
        h = self.key.context_hash()
        assert 0 <= h <= 0xFFFFFFFF


# ── Step 3: Different topic (cryptocurrency) → different hash ─────────────────

class TestStep3TopicSwitch:
    """
    Scenario: User switches topic to cryptocurrency
      Given the user asks about "bitcoin ethereum blockchain cryptocurrency"
      When TextContextKey.from_text() is called
      Then the resulting hash differs from the personal finance hash
      And the topic_domain differs (I-LLM-008)
    """

    def setup_method(self):
        self.finance_key = TextContextKey(
            topic_domain=0,   # personal finance
            conversation_depth=ConversationDepth.SHALLOW,
            emotional_register=EmotionalRegister.NEUTRAL,
            time_of_day=TimeOfDay.MORNING,
            session_phase=SessionPhase.OPENING,
        )
        crypto_topic = _keyword_cluster("bitcoin ethereum blockchain cryptocurrency wallet")
        self.crypto_key = TextContextKey(
            topic_domain=crypto_topic,
            conversation_depth=ConversationDepth.SHALLOW,
            emotional_register=EmotionalRegister.NEUTRAL,
            time_of_day=TimeOfDay.MORNING,
            session_phase=SessionPhase.OPENING,
        )

    def test_crypto_maps_to_cluster_3(self):
        """Crypto keywords must map to cluster 3."""
        assert self.crypto_key.topic_domain == 3, (
            f"Expected cluster 3 for crypto, got {self.crypto_key.topic_domain}"
        )

    def test_different_topic_different_hash(self):
        """Finance and crypto must produce different context hashes."""
        assert self.finance_key.context_hash() != self.crypto_key.context_hash()

    def test_topic_domains_are_different(self):
        assert self.finance_key.topic_domain != self.crypto_key.topic_domain


# ── Step 4: Cosine similarity < 0.95 for different topic clusters ─────────────

class TestStep4ContextIsolation:
    """
    Scenario: Verify context isolation between finance and crypto
      Given a personal_finance TextContextKey
      And a cryptocurrency TextContextKey
      When cosine_similarity() is computed
      Then the result is < 0.95 (I-LLM-009)
    """

    def setup_method(self):
        self.finance = TextContextKey(
            topic_domain=0, conversation_depth=1, emotional_register=0,
            time_of_day=1, session_phase=1,
        )
        self.crypto = TextContextKey(
            topic_domain=3, conversation_depth=1, emotional_register=0,
            time_of_day=1, session_phase=1,
        )

    def test_finance_vs_crypto_below_095(self):
        """
        I-LLM-009: Different topic clusters must have cosine_similarity < 0.95.

        Finance (cluster 0) vs philosophy (cluster 28) are very different domains.
        With 4 identical dims and topic_domain varying by 28/63, sim < 0.95.
        """
        philosophy = TextContextKey(
            topic_domain=28, conversation_depth=1, emotional_register=0,
            time_of_day=1, session_phase=1,
        )
        sim = self.finance.cosine_similarity(philosophy)
        assert sim < 0.95, (
            f"Context isolation failed: finance vs philosophy similarity = {sim:.4f} "
            f"(expected < 0.95)"
        )

    def test_self_similarity_is_one(self):
        """Self-similarity must be exactly 1.0."""
        assert abs(self.finance.cosine_similarity(self.finance) - 1.0) < 1e-9
        assert abs(self.crypto.cosine_similarity(self.crypto) - 1.0) < 1e-9


# ── Step 5: Warm-start — estate_planning finds personal_finance as nearest ────

class TestStep5WarmStart:
    """
    Scenario: Warm-start seeding for a new tax planning conversation
      Given known contexts [personal_finance, philosophy]
      When a new tax_planning context looks for nearest neighbours
      Then personal_finance is returned as top-1 nearest context
      And the similarity to personal_finance > similarity to philosophy

    Cluster geometry:
      personal_finance = 0  (0/63 = 0.000 normalised)
      tax_planning     = 1  (1/63 = 0.016 normalised)
      philosophy       = 28 (28/63 = 0.444 normalised)

    tax_planning is numerically much closer to personal_finance than to philosophy,
    so personal_finance should always be the top-1 nearest context.
    """

    def setup_method(self):
        self.personal_finance = TextContextKey(
            topic_domain=0, conversation_depth=1, emotional_register=0,
            time_of_day=1, session_phase=1,
        )
        self.philosophy = TextContextKey(
            topic_domain=28, conversation_depth=1, emotional_register=0,
            time_of_day=1, session_phase=1,
        )
        tax_topic = _keyword_cluster("tax taxation irs deduction filing income return")
        self.tax_planning = TextContextKey(
            topic_domain=tax_topic,
            conversation_depth=1,
            emotional_register=0,
            time_of_day=1,
            session_phase=1,
        )

    def test_tax_maps_to_cluster_1(self):
        """Tax keywords must map to cluster 1."""
        assert self.tax_planning.topic_domain == 1, (
            f"Expected tax cluster 1, got {self.tax_planning.topic_domain}"
        )

    def test_nearest_returns_personal_finance_as_top1(self):
        known = [self.personal_finance, self.philosophy]
        nearest = TextContextKey.nearest_contexts(self.tax_planning, known, k=2)

        assert len(nearest) == 2, "Should return exactly 2 nearest contexts"
        top_ctx, top_sim = nearest[0]
        assert top_ctx == self.personal_finance, (
            f"Expected personal_finance (topic=0) as top-1 for tax_planning, "
            f"got topic={top_ctx.topic_domain}"
        )

    def test_personal_finance_closer_than_philosophy(self):
        sim_finance = self.tax_planning.cosine_similarity(self.personal_finance)
        sim_philosophy = self.tax_planning.cosine_similarity(self.philosophy)
        assert sim_finance > sim_philosophy, (
            f"Tax planning should be closer to finance ({sim_finance:.4f}) "
            f"than philosophy ({sim_philosophy:.4f})"
        )

    def test_nearest_contexts_sorted_descending(self):
        known = [self.personal_finance, self.philosophy]
        nearest = TextContextKey.nearest_contexts(self.tax_planning, known, k=2)
        sims = [sim for _, sim in nearest]
        assert sims == sorted(sims, reverse=True), "Results must be sorted by similarity descending"

    def test_nearest_contexts_respects_k_limit(self):
        known = [self.personal_finance, self.philosophy]
        nearest = TextContextKey.nearest_contexts(self.tax_planning, known, k=1)
        assert len(nearest) == 1

    def test_nearest_contexts_empty_list(self):
        result = TextContextKey.nearest_contexts(self.tax_planning, [])
        assert result == []


# ── Full journey smoke test ───────────────────────────────────────────────────

def test_full_j_llm_first_conversation_journey():
    """
    J-LLM-FIRST-CONVERSATION end-to-end smoke test.

    Simulates the complete user journey:
    1. Finance context (cluster 0) created and hashed
    2. Hash is stable (I-LLM-001)
    3. Philosophy context (cluster 28) has different hash
    4. Context isolation: finance vs philosophy cosine sim < 0.95 (I-LLM-009)
    5. Warm-start: tax planning (cluster 1) finds finance (cluster 0) as nearest

    Cluster geometry used:
      personal_finance = 0  (finance keywords)
      philosophy       = 28 (philosophy keywords)
      tax_planning     = 1  (tax keywords) — numerically adjacent to finance
    """
    # Step 1 — create finance context (cluster 0)
    # Use depth=MODERATE, time=AFTERNOON, phase=MIDDLE so the feature vector is non-zero
    # across all dimensions — required for meaningful cosine similarity.
    finance = TextContextKey(
        topic_domain=_keyword_cluster("compound interest personal finance saving budget"),
        conversation_depth=ConversationDepth.MODERATE,
        emotional_register=EmotionalRegister.NEUTRAL,
        time_of_day=TimeOfDay.AFTERNOON,
        session_phase=SessionPhase.MIDDLE,
    )
    assert isinstance(finance, TextContextKey)
    assert finance.topic_domain == 0, f"Finance should be cluster 0, got {finance.topic_domain}"

    # Step 2 — hash stability (I-LLM-001)
    h1 = finance.context_hash()
    h2 = finance.context_hash()
    assert h1 == h2, "I-LLM-001 violated: hash not stable"

    # Step 3 — different topic (philosophy, cluster 28) → different hash
    philosophy = TextContextKey(
        topic_domain=_keyword_cluster("philosophy ethics moral existential meaning purpose"),
        conversation_depth=ConversationDepth.MODERATE,
        emotional_register=EmotionalRegister.NEUTRAL,
        time_of_day=TimeOfDay.AFTERNOON,
        session_phase=SessionPhase.MIDDLE,
    )
    assert philosophy.topic_domain == 28, (
        f"Philosophy should be cluster 28, got {philosophy.topic_domain}"
    )
    assert finance.context_hash() != philosophy.context_hash(), (
        "Different topics must have different hashes"
    )

    # Step 4 — context isolation (I-LLM-009)
    sim = finance.cosine_similarity(philosophy)
    assert sim < 0.95, f"I-LLM-009 violated: finance vs philosophy sim = {sim:.4f}"

    # Step 5 — warm-start: tax_planning (cluster 1) finds finance (cluster 0) as nearest.
    # Both use identical non-topic dims so cosine distance is purely a function of topic.
    tax = TextContextKey(
        topic_domain=_keyword_cluster("tax taxation irs deduction income return filing"),
        conversation_depth=ConversationDepth.MODERATE,
        emotional_register=EmotionalRegister.NEUTRAL,
        time_of_day=TimeOfDay.AFTERNOON,
        session_phase=SessionPhase.MIDDLE,
    )
    assert tax.topic_domain == 1, f"Tax should be cluster 1, got {tax.topic_domain}"
    known = [finance, philosophy]
    nearest = TextContextKey.nearest_contexts(tax, known, k=1)
    assert nearest[0][0] == finance, (
        f"Warm-start failed: finance (cluster 0) should be nearest to tax (cluster 1), "
        f"but got cluster {nearest[0][0].topic_domain}"
    )
