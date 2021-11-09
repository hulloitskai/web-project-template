module.exports = {
  async up(db) {
    const user = db.collection("user");
    await user.createIndex({ email: 1 }, { name: "email" });
    await user.createIndex({ phone: 1 }, { name: "phone" });
  },

  async down(db) {
    const user = db.collection("user");
    await user.dropIndex(["email", "phone"]);
  },
};
