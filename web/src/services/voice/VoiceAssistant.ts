/**
 * Voice Assistant Main Controller
 *
 * Orchestrates voice input, user identification, command processing,
 * and integration with autonomous behavior and learning engines
 */

import { VoiceProfileService } from './VoiceProfileService';
import { PersonalBriefingService } from '../briefing/PersonalBriefingService';
import { ConversationMemoryService } from '../memory/ConversationMemoryService';
import { VoiceIdentification, CommandResult } from '../../types/voice';

export class VoiceAssistant {
  private voiceService: VoiceProfileService;
  private briefingService: PersonalBriefingService;
  private memoryService: ConversationMemoryService;
  private recognition: SpeechRecognition | null = null;
  private isListening: boolean = false;

  constructor() {
    this.voiceService = new VoiceProfileService();
    this.briefingService = new PersonalBriefingService();
    this.memoryService = new ConversationMemoryService();

    // Initialize speech recognition
    if (typeof window !== 'undefined') {
      const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
      if (SpeechRecognition) {
        this.recognition = new SpeechRecognition();
        this.recognition.continuous = false;
        this.recognition.interimResults = false;
        this.recognition.lang = 'en-US';
      }
    }
  }

  /**
   * Handle voice input (main flow)
   */
  async handleVoiceInput(audioData: ArrayBuffer): Promise<void> {
    try {
      // Step 1: Identify user by voice
      const identification = await this.identifyUser(audioData);

      if (!identification.userId || identification.confidence < 0.85) {
        // Anonymous mode
        await this.handleAnonymousMode();
        return;
      }

      // Step 2: Transcribe audio to text
      const text = await this.transcribeAudio(audioData);

      // Step 3: Process command or query
      await this.processCommand(identification.userId, text);

      // Step 4: Store interaction
      await this.observeInteraction(identification.userId, text, 'processed', true);

    } catch (error) {
      console.error('Voice input processing failed:', error);
    }
  }

  /**
   * Identify user by voice biometrics
   * Contract: I-VOICE-001 (Speaker Identification ≥85%)
   */
  async identifyUser(audioData: ArrayBuffer): Promise<VoiceIdentification> {
    return this.voiceService.identifyUser(audioData);
  }

  /**
   * Transcribe audio to text using external STT service
   */
  async transcribeAudio(audioData: ArrayBuffer): Promise<string> {
    // Mock implementation - would call Whisper API or similar
    console.log('[STT] Transcribing audio...');

    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 500));

    return 'Good morning'; // Mock transcription
  }

  /**
   * Process voice command or query
   */
  async processCommand(userId: string, text: string): Promise<void> {
    const lowerText = text.toLowerCase();

    // Morning briefing trigger
    if (lowerText.includes('good morning') ||
        lowerText.includes('what\'s new') ||
        lowerText.includes('daily update')) {
      await this.triggerMorningBriefing(userId);
      return;
    }

    // News request
    if (lowerText.includes('news') || lowerText.includes('headlines')) {
      await this.deliverNews(userId);
      return;
    }

    // Email request
    if (lowerText.includes('email') || lowerText.includes('messages')) {
      await this.deliverEmail(userId);
      return;
    }

    // Memory recall
    if (lowerText.includes('yesterday') ||
        lowerText.includes('what did') ||
        lowerText.includes('we talked about')) {
      await this.deliverMemoryRecall(userId);
      return;
    }

    // General conversation
    const response = await this.generateResponse(userId, text);
    await this.speakResponse(response);
  }

  /**
   * Generate conversational response
   */
  async generateResponse(userId: string, text: string): Promise<string> {
    // Store conversation turn
    await this.memoryService.storeConversationTurn({
      speaker: 'user',
      text,
      timestamp: Date.now()
    }, userId);

    // Generate response (mock - would use LLM)
    const response = `I heard you say: ${text}. How can I help?`;

    // Store bot response
    await this.memoryService.storeConversationTurn({
      speaker: 'mbot',
      text: response,
      timestamp: Date.now()
    }, userId);

    return response;
  }

  /**
   * Speak response using TTS
   */
  async speakResponse(text: string): Promise<void> {
    await this.briefingService.speakBriefing(text);
  }

  /**
   * Trigger morning briefing
   * Contract: I-VOICE-003 (Personalized Content)
   */
  async triggerMorningBriefing(userId: string): Promise<void> {
    const briefing = await this.briefingService.generateBriefing(userId);
    await this.briefingService.deliverBriefing(userId, briefing);
  }

  /**
   * Deliver news only
   */
  async deliverNews(userId: string): Promise<void> {
    const briefing = await this.briefingService.generateBriefing(userId);
    const newsSection = briefing.sections.find(s => s.type === 'news');

    if (newsSection && newsSection.content) {
      await this.speakResponse(newsSection.content);
    } else {
      await this.speakResponse('I don\'t have any news for you right now.');
    }
  }

  /**
   * Deliver email summary
   */
  async deliverEmail(userId: string): Promise<void> {
    const briefing = await this.briefingService.generateBriefing(userId);
    const emailSection = briefing.sections.find(s => s.type === 'email');

    if (emailSection && emailSection.content) {
      await this.speakResponse(emailSection.content);
    } else {
      await this.speakResponse('No email accounts connected.');
    }
  }

  /**
   * Deliver memory recall
   * Contract: I-MEMORY-001 (Activity Tracking)
   */
  async deliverMemoryRecall(userId: string): Promise<void> {
    const briefing = await this.briefingService.generateBriefing(userId);
    const memorySection = briefing.sections.find(s => s.type === 'memory');

    if (memorySection && memorySection.content) {
      await this.speakResponse(memorySection.content);
    } else {
      await this.speakResponse('I don\'t recall what we talked about recently.');
    }
  }

  /**
   * Handle anonymous mode (unknown user)
   * Contract: I-VOICE-006 (Graceful Fallback)
   */
  async handleAnonymousMode(): Promise<void> {
    await this.speakResponse('Hello! I don\'t think we\'ve met. I\'m mBot! Would you like to set up your voice profile?');
  }

  /**
   * Check in after inactivity (autonomous behavior integration)
   */
  async checkInAfterInactivity(userId: string): Promise<void> {
    const questions = await this.memoryService.getFollowUpQuestions(userId);

    if (questions.length > 0) {
      const topQuestion = questions
        .filter(q => !q.answered && q.validUntil > Date.now())
        .sort((a, b) => b.priority - a.priority)[0];

      if (topQuestion) {
        await this.speakResponse(topQuestion.question);
      }
    }
  }

  /**
   * Observe interaction for learning
   * Integration with LearningEngine (#92)
   */
  async observeInteraction(
    userId: string,
    input: string,
    response: string,
    success: boolean
  ): Promise<void> {
    // Store in conversation memory
    await this.memoryService.storeConversationTurn({
      speaker: 'user',
      text: input,
      timestamp: Date.now()
    }, userId);

    await this.memoryService.storeConversationTurn({
      speaker: 'mbot',
      text: response,
      timestamp: Date.now()
    }, userId);

    // TODO: Send to LearningEngine for preference learning
    console.log(`[Learning] Interaction recorded: success=${success}`);
  }

  /**
   * Start continuous listening
   */
  startListening(callback: (text: string) => void): void {
    if (!this.recognition) {
      console.error('Speech recognition not available');
      return;
    }

    this.recognition.onresult = (event) => {
      const transcript = event.results[0][0].transcript;
      callback(transcript);
    };

    this.recognition.onerror = (event) => {
      console.error('Speech recognition error:', event.error);
      this.isListening = false;
    };

    this.recognition.onend = () => {
      this.isListening = false;
    };

    this.recognition.start();
    this.isListening = true;
  }

  /**
   * Stop listening
   */
  stopListening(): void {
    if (this.recognition && this.isListening) {
      this.recognition.stop();
      this.isListening = false;
    }
  }

  /**
   * Check if currently listening
   */
  isCurrentlyListening(): boolean {
    return this.isListening;
  }
}
