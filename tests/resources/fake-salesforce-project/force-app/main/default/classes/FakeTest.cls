@IsTest
public with sharing class FakeTest {
    @IsTest
    static void itPasses() {
        System.Assert.areEqual(2, 2);
      }

    @IsTest
    static void itFails() {
        System.Assert.areEqual(2, 3);
      }
}